// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::handle::{HandleInstanceId, MethodId};
use crate::queue::Queue;
use cbsb::ipc::{IpcRecv, IpcSend, RecvFlag, Terminate};
use crossbeam::channel::{bounded, Receiver, Sender};
use std::io::Cursor;
use std::sync::Arc;
use std::thread;

type SlotId = u32;
type HandlerId = u32;
type Dispatcher = dyn Fn(Cursor<&mut Vec<u8>>, HandleInstanceId, MethodId, &[u8]) + Send + Sync;

const SLOT_CALL_OR_RETURN_INDICATOR: u32 = 1024 * 1024;
#[cfg(debug_assertions)]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100_000);
#[cfg(not(debug_assertions))]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);

// In A's view...
// A calls another module:
// => A becomes a Client, and makes invocation of an outbound call.
// A called another module, and the another module responds:
// => A is still a Client, and receives a response of outbound call.
// Another module calls A:
// => A becomes a Service, and receives an invocation of an inbound call.
// Another module called A, and A responds:
// => A is still a Service, and makes a response of an inbound call.

#[derive(PartialEq, Debug)]
pub struct PacketHeader {
    pub slot: SlotId,
    pub handle: HandleInstanceId,
    pub method: MethodId,
}

impl PacketHeader {
    pub fn new(buffer: &[u8]) -> Self {
        unsafe { std::ptr::read(buffer.as_ptr().cast()) }
    }

    pub fn write(&self, buffer: &mut [u8]) {
        unsafe {
            std::ptr::copy_nonoverlapping(self, buffer.as_mut_ptr().cast(), 1);
        }
    }
}

#[test]
fn encoding_packet_header() {
    let ph1 = PacketHeader {
        slot: 0x1234,
        handle: HandleInstanceId {
            trait_id: 0x9999,
            index: 0x8888,
        },
        method: 0x5678,
    };
    let mut buffer = vec![0 as u8; std::mem::size_of::<PacketHeader>()];
    ph1.write(&mut buffer);
    let ph2 = PacketHeader::new(&buffer);
    assert_eq!(ph2, ph1);
}

fn ipc_sender<S: IpcSend>(queue_end: Receiver<Vec<u8>>, send: S) {
    loop {
        let data = queue_end.recv().unwrap();
        if data.is_empty() {
            break
        }
        if data.len() < std::mem::size_of::<PacketHeader>() {
            panic!("Invalid packet received: {:?}", data);
        }
        send.send(&data);
    }
}

fn service_handler(
    id: HandlerId,
    invoke: Receiver<Vec<u8>>,
    response: Sender<Vec<u8>>,
    token: &Queue<HandlerId>,
    dispatcher: &Dispatcher,
) {
    loop {
        let data = invoke.recv().unwrap();
        if data.is_empty() {
            break
        }
        if data.len() < std::mem::size_of::<PacketHeader>() {
            panic!("Invalid packet received: {:?}", data);
        }
        let mut header = PacketHeader::new(&data);
        let mut buffer: Vec<u8> = vec![0; std::mem::size_of::<PacketHeader>()];
        dispatcher(
            {
                let mut c = Cursor::new(&mut buffer);
                c.set_position(std::mem::size_of::<PacketHeader>() as u64);
                c
            },
            header.handle,
            header.method,
            &data,
        );

        header.slot += SLOT_CALL_OR_RETURN_INDICATOR;
        header.write(&mut buffer);

        response.send(buffer).unwrap();
        token.push(id);
    }
}

/// CallSlot represents an instance of concurrent call to the another module
struct CallSlot {
    id: SlotId,
    invoke: Sender<Vec<u8>>,
    response: Receiver<Vec<u8>>,
}

/// Internal state of server. This is immutable during the main loop.
pub struct ServerInternal {
    // Configurations. Bigger means being more capable of concurrency
    ipc_channel_capcity: usize,
    client_slots_size: usize,
    service_handlers_size: usize,

    client_slots: Arc<Queue<CallSlot>>,
    client_response_sender: Vec<Sender<Vec<u8>>>,

    service_handlers: Vec<thread::JoinHandle<()>>,
    service_handlers_invoke: Vec<Sender<Vec<u8>>>,
    service_handlers_token: Arc<Queue<HandlerId>>,

    dispatcher: Arc<Dispatcher>,
}

impl ServerInternal {
    pub fn new(
        ipc_channel_capcity: usize,
        client_slots_size: usize,
        service_handlers_size: usize,
        dispatcher: Arc<Dispatcher>,
    ) -> Self {
        ServerInternal {
            ipc_channel_capcity,
            client_slots_size,
            service_handlers_size,
            client_slots: Arc::new(Queue::new(client_slots_size)),
            client_response_sender: Vec::new(),
            service_handlers: Vec::new(),
            service_handlers_invoke: Vec::new(),
            service_handlers_token: Arc::new(Queue::new(service_handlers_size)),
            dispatcher,
        }
    }
}

pub fn main_routine_common1<S: IpcSend + 'static>(
    server: &mut ServerInternal,
    send: S,
) -> (thread::JoinHandle<()>, Sender<Vec<u8>>) {
    let (ipc_sender_sender, ipc_sender_receiver) = bounded(server.ipc_channel_capcity);
    for i in 0..server.client_slots_size {
        let (call_sender, call_receiver) = bounded(1);
        server.client_slots.push(CallSlot {
            id: i as u32,
            invoke: ipc_sender_sender.clone(),
            response: call_receiver,
        });
        server.client_response_sender.push(call_sender);
    }
    let sender_thread = thread::spawn(move || {
        ipc_sender(ipc_sender_receiver, send);
    });

    for i in 0..server.service_handlers_size {
        let (invoke_sender, invoke_receiver) = bounded(1);
        server.service_handlers_token.push(i as HandlerId);
        server.service_handlers_invoke.push(invoke_sender);

        let sender = ipc_sender_sender.clone();
        let token = server.service_handlers_token.clone();
        let dispatcher = server.dispatcher.clone();
        server.service_handlers.push(thread::spawn(move || {
            let d = dispatcher;
            service_handler(i as u32, invoke_receiver, sender, &token, &*d);
        }));
    }
    (sender_thread, ipc_sender_sender)
}

/// Unlikely to ~~~ this takes &Server, not &mut Server, which facilitates the Rust concurrency.
pub fn main_routine_common2<R: IpcRecv + 'static>(
    server: &ServerInternal,
    ipc_sender_sender: Sender<Vec<u8>>,
    recv: R,
) {
    loop {
        let data = match recv.recv(None) {
            Err(RecvFlag::TimeOut) => panic!(),
            Err(RecvFlag::Termination) => {
                ipc_sender_sender.send([].to_vec()).unwrap();
                return
            }
            Ok(x) => x,
        };
        let header = PacketHeader::new(&data);
        let (slot, is_inbound_call) = if header.slot >= SLOT_CALL_OR_RETURN_INDICATOR {
            (header.slot - SLOT_CALL_OR_RETURN_INDICATOR, false)
        } else {
            (header.slot, true)
        };

        if is_inbound_call {
            let token = server.service_handlers_token.pop(Some(TIMEOUT)).expect("Module doesn't respond");
            server.service_handlers_invoke[token as usize].send(data).unwrap();
        } else {
            // response of outbound call
            server.client_response_sender[slot as usize].send(data).unwrap();
        }
    }
}

pub fn main_routine_common3(server: &mut ServerInternal, sender_thread: thread::JoinHandle<()>) {
    sender_thread.join().unwrap();

    // Reserve all the tokens, to check that there is no callback handling now.
    for _ in 0..server.service_handlers_size {
        server.service_handlers_token.pop(Some(TIMEOUT)).expect("Module is not ready to terminate");
    }

    for i in 0..server.service_handlers_size {
        server.service_handlers_invoke[i].send([].to_vec()).unwrap();
    }

    while let Some(handler) = server.service_handlers.pop() {
        handler.join().unwrap();
    }
}

/// A server communicating with IPC. This will be provided per link.
pub struct Server {
    raw: Arc<ServerInternal>,
    main_thread: Option<thread::JoinHandle<()>>,
    sender_thread: Option<thread::JoinHandle<()>>,
    termiantor: Option<Box<dyn Terminate>>,
}
impl Server {
    pub fn new<S: IpcSend + 'static, R: IpcRecv + 'static>(mut server: ServerInternal, send: S, recv: R) -> Self {
        let (sender_thread, ipc_sender_sender) = main_routine_common1(&mut server, send);
        let server2 = Arc::new(server);
        let server3 = server2.clone();
        let terminator = recv.create_terminate();

        // This will return after terminate()
        let main_thread = thread::spawn(move || main_routine_common2(&server3, ipc_sender_sender, recv));
        Server {
            raw: server2,
            main_thread: Some(main_thread),
            sender_thread: Some(sender_thread),
            termiantor: Some(Box::new(terminator)),
        }
    }

    /// Call from this module to another moudle
    pub fn call(&self, handle: HandleInstanceId, method: MethodId, mut data: Vec<u8>) -> Vec<u8> {
        let slot = self.raw.client_slots.pop(Some(TIMEOUT)).expect("Module doesn't respond");
        let mut header = PacketHeader::new(&data);
        header.handle = handle;
        header.method = method;
        header.slot = slot.id as u32;
        header.write(&mut data);
        slot.invoke.send(data).unwrap();
        let return_value = slot.response.recv().unwrap();
        self.raw.client_slots.push(slot); //return back
        return_value
    }

    /// Terminate itself immediately.
    pub fn terminate(&mut self) {
        self.termiantor.take().unwrap().terminate();
        self.main_thread.take().unwrap().join().unwrap();
        main_routine_common3(Arc::get_mut(&mut self.raw).unwrap(), self.sender_thread.take().unwrap());
    }
}

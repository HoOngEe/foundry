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

use super::*;
use crossbeam::channel::{bounded, Receiver, RecvTimeoutError, Sender};
use std::collections::hash_map::HashMap;
use std::sync::Mutex;

// This acts like an IPC, but actually in-process communication.
// It will be useful when you have to simulate IPC, but the two ends don't have
// to be actually in separated processes.

lazy_static! {
    static ref POOL: Mutex<HashMap<String, (Sender<Vec<u8>>, Receiver<Vec<u8>>)>> = { Mutex::new(HashMap::new()) };
}

pub struct SameProcessLinker {
    key_server: String,
    key_client: String,
}

impl TwoWayInitialize for SameProcessLinker {
    type Server = SameProcess;
    type Client = SameProcess;

    fn new(_name: String) -> Self {
        let key_server = generate_random_name();
        let key_client = generate_random_name();

        let (send1, recv1) = bounded(256);
        let (send2, recv2) = bounded(256);

        POOL.lock().unwrap().insert(key_server.clone(), (send1, recv2));
        POOL.lock().unwrap().insert(key_client.clone(), (send2, recv1));

        SameProcessLinker {
            key_server,
            key_client,
        }
    }

    fn create(&self) -> (Vec<u8>, Vec<u8>) {
        (serde_cbor::to_vec(&self.key_server).unwrap(), serde_cbor::to_vec(&self.key_client).unwrap())
    }
}

impl Drop for SameProcessLinker {
    fn drop(&mut self) {
        let map = POOL.lock().unwrap();
        assert!(map.get(&self.key_server).is_none());
        assert!(map.get(&self.key_client).is_none());
    }
}

pub struct SameProcessSend(Sender<Vec<u8>>);

impl IpcSend for SameProcessSend {
    fn send(&self, data: &[u8]) {
        self.0.send(data.to_vec()).unwrap()
    }
}

pub struct SameProcessRecv(Receiver<Vec<u8>>, Sender<Vec<u8>>);

pub struct Terminator(Sender<Vec<u8>>);

impl Terminate for Terminator {
    fn terminate(&self) {
        self.0.send([].to_vec()).unwrap();
    }
}

impl IpcRecv for SameProcessRecv {
    type MyTerminate = Terminator;

    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvFlag> {
        let x = if let Some(t) = timeout {
            self.0.recv_timeout(t).map_err(|e| {
                if e == RecvTimeoutError::Timeout {
                    RecvFlag::TimeOut
                } else {
                    panic!()
                }
            })
        } else {
            Ok(self.0.recv().unwrap())
        }?;

        if x.len() == 0 {
            return Err(RecvFlag::Termination)
        }
        Ok(x)
    }

    fn create_terminate(&self) -> Self::MyTerminate {
        Terminator(self.1.clone())
    }
}

pub struct SameProcess {
    send: SameProcessSend,
    recv: SameProcessRecv,
}

impl InterProcessUnit for SameProcess {
    fn new(data: Vec<u8>) -> Self {
        let key: String = serde_cbor::from_slice(&data).unwrap();
        let (send, recv) = POOL.lock().unwrap().remove(&key).unwrap();
        SameProcess {
            send: SameProcessSend(send.clone()),
            recv: SameProcessRecv(recv, send),
        }
    }
    fn ready(&mut self) {}
}

impl IpcSend for SameProcess {
    fn send(&self, data: &[u8]) {
        self.send.send(data)
    }
}

impl IpcRecv for SameProcess {
    type MyTerminate = Terminator;

    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvFlag> {
        self.recv.recv(timeout)
    }

    fn create_terminate(&self) -> Self::MyTerminate {
        self.recv.create_terminate()
    }
}

impl Ipc for SameProcess {
    type SendOnly = SameProcessSend;
    type RecvOnly = SameProcessRecv;

    fn split(self) -> (Self::SendOnly, Self::RecvOnly) {
        (self.send, self.recv)
    }
}

impl TwoWayInitializableIpc for SameProcess {
    type Linker = SameProcessLinker;
}

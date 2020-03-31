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

use crate::context::{Config, Context, Custom, PortTable};
use crate::handle::{Dispatcher, HandlePreset, ImportedHandle, MethodId};
use crate::port::server::PacketHeader;
use crate::port::Port;
use crate::port::PortId;
use crate::IpcBase;
use cbsb::execution::executee;
use cbsb::ipc::{domain_socket, same_process, InterProcessUnit, Ipc, IpcRecv, IpcSend};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn recv<I: Ipc, T: serde::de::DeserializeOwned>(ctx: &executee::Context<I>) -> T {
    serde_cbor::from_slice(&ctx.ipc.as_ref().unwrap().recv(None).unwrap()).unwrap()
}

pub fn send<I: Ipc, T: serde::Serialize>(ctx: &executee::Context<I>, data: &T) {
    ctx.ipc.as_ref().unwrap().send(&serde_cbor::to_vec(data).unwrap());
}

fn create_port<D: Dispatcher + 'static>(
    port_id: PortId,
    ipc_type: Vec<u8>,
    ipc_config: Vec<u8>,
    dispatcher: Arc<D>,
) -> Port<D> {
    let ipc_type: String = serde_cbor::from_slice(&ipc_type).unwrap();

    if ipc_type == "DomainSocket" {
        let ipc = domain_socket::DomainSocket::new(ipc_config);
        let (send, recv) = ipc.split();
        Port::new(port_id, send, recv, dispatcher)
    } else if ipc_type == "SameProcess" {
        let ipc = same_process::SameProcess::new(ipc_config);
        let (send, recv) = ipc.split();
        Port::new(port_id, send, recv, dispatcher)
    } else {
        panic!("Invalid port creation request");
    }
}

pub fn core<I: Ipc, C: Custom, D: Dispatcher + 'static, H: HandlePreset>(
    args: Vec<String>,
    handle_preset: &mut H,
    context_setter: Box<dyn Fn(Context<C, D>) -> ()>,
) {
    let ctx = executee::start::<I>(args);

    let config = recv(&ctx);
    let custom = C::new(&config);
    let ports: Arc<Mutex<HashMap<PortId, (Config, Port<D>)>>> = Arc::new(Mutex::new(HashMap::new()));
    let global_context = Context::new(ports.clone(), config, custom);
    context_setter(global_context);

    loop {
        let message: String = recv(&ctx);
        if message == "link" {
            let (port_id, port_config, ipc_type, ipc_config) = recv(&ctx);
            let dispather = Arc::new(D::new(port_id, 128));
            ports.lock().unwrap().insert(port_id, (port_config, create_port(port_id, ipc_type, ipc_config, dispather)));
        } else if message == "unlink" {
            let (port_id,) = recv(&ctx);
            ports.lock().unwrap().remove(&port_id).unwrap();
        } else if message == "terminate" {
            break
        } else if message == "handle_export" {
            // export a default, preset handles for a specific port
            let (port_id,) = recv(&ctx);
            send(&ctx, &handle_preset.export(port_id).unwrap());
        } else if message == "handle_import" {
            // import a default, preset handles for a specific port
            let (handle,) = recv(&ctx);
            handle_preset.import(handle).unwrap();
        } else if message == "call" {
            // debug purpose direct handle call
            let (handle, method, mut data): (ImportedHandle, u32, Vec<u8>) = recv(&ctx);
            let mut packet = vec![0 as u8; std::mem::size_of::<PacketHeader>()];
            packet.append(&mut data);
            let port_table = ports.lock().unwrap();
            println!("QWEQWEQWE");
            let result = port_table.get(&handle.port_id).unwrap().1.call(handle.id, method, packet);
            send(&ctx, &result);
        } else {
            panic!("Unexpected message")
        }
        send(&ctx, &"done".to_owned());
    }

    ctx.terminate();
}

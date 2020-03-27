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

use crate::context::{Config, Context, Custom};
use crate::handle::Dispatcher;
use crate::port::Port;
use crate::port::PortId;
use crate::IpcBase;
use cbsb::execution::executee;
use cbsb::ipc::domain_socket::DomainSocket;
use cbsb::ipc::{InterProcessUnit, IpcRecv, IpcSend};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn recv<T: serde::de::DeserializeOwned>(ctx: &executee::Context<IpcBase>) -> T {
    serde_cbor::from_slice(&ctx.ipc.as_ref().unwrap().recv(None).unwrap()).unwrap()
}

pub fn send<T: serde::Serialize>(ctx: &executee::Context<IpcBase>, data: &T) {
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
        let ipc = DomainSocket::new(ipc_config);
        let ipc_send = ipc.create_sender();
        Port::new(port_id, ipc_send, ipc, dispatcher)
    } else {
        panic!("Invalid port creation request");
    }
}

pub fn core<C: Custom, D: Dispatcher + 'static>() {
    let ctx = executee::start::<crate::IpcBase>();

    // FIXME: Does rust guarantee left-to-right evaluation order in the struct initialization?
    let kind: String = recv(&ctx);
    let id: String = recv(&ctx);
    let key: String = recv(&ctx);
    let args: Vec<u8> = recv(&ctx);
    let config = Config {
        kind,
        id,
        key,
        args,
    };
    let custom = C::new(&config);
    let ports: Arc<Mutex<HashMap<PortId, Port<D>>>> = Arc::new(Mutex::new(HashMap::new()));
    let global_context = Context::new(ports.clone(), config, custom);

    let mut ports: HashMap<PortId, Port<D>> = HashMap::new();

    loop {
        let message: String = recv(&ctx);
        if message == "connect" {
            let port_id = recv(&ctx);
            let arg1 = recv(&ctx);
            let arg2 = recv(&ctx);
            let dispather = Arc::new(D::new(port_id, 128));
            ports.insert(port_id, create_port(port_id, arg1, arg2, dispather));
        } else if message == "disconnect" {
            let port_id = recv(&ctx);
            ports.remove(&port_id).unwrap();
        } else if message == "terminate" {
            break
        } else if message == "handle" {
        } else {
            panic!("Unexpected message")
        }
        send(&ctx, &"done".to_owned());
    }
}

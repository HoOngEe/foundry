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

pub mod server;

use crate::handle::{Dispatcher, HandleInstanceId, MethodId};
use cbsb::ipc::{IpcRecv, IpcSend};
use std::sync::{Arc, Mutex};

pub type PortId = usize;

pub struct Port<D: Dispatcher> {
    dispatcher: Arc<D>,
    server: server::Server,
}

impl<D: Dispatcher + 'static> Port<D> {
    pub fn new<S: IpcSend + 'static, R: IpcRecv + 'static>(
        port_id: PortId,
        send: S,
        recv: R,
        dispatcher: Arc<D>,
    ) -> Self {
        let dispatcher_clone = dispatcher.clone();
        let server = server::Server::new(
            port_id,
            server::ServerInternal::new(
                128,
                128,
                128,
                Arc::new(move |buffer: &mut [u8], handle: HandleInstanceId, method: MethodId, data: &[u8]| {
                    dispatcher_clone.dispatch_and_call(buffer, handle, method, data)
                }),
            ),
            send,
            recv,
        );
        Port {
            dispatcher,
            server,
        }
    }

    pub fn call(&self, handle: HandleInstanceId, method: MethodId, data: Vec<u8>) -> Vec<u8> {
        self.server.call(handle, method, data)
    }

    pub fn dispatcher_get(&self) -> Arc<D> {
        self.dispatcher.clone()
    }
}

impl<D: Dispatcher> Drop for Port<D> {
    fn drop(&mut self) {
        self.server.terminate();
    }
}

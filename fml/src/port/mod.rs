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

mod server;

use crate::handle::{HandleInstanceId, MethodId, Dispatcher};
use cbsb::ipc::{IpcRecv, IpcSend};
use std::sync::{Arc, Mutex};

pub struct Port<D: Dispatcher>  {
    dispatcher: Arc<D>,
    server: server::Server,
}

impl<D: Dispatcher + 'static> Port<D> {
    pub fn new<S: IpcSend + 'static, R: IpcRecv + 'static>(name: &str, send: S, recv: R, dispatcher: Arc<D>) -> Self {
        let dispatcher_clone = dispatcher.clone();
        let server = server::Server::new(
            server::ServerInternal::new(
                128,
                128,
                128,
                Arc::new(move |handle: HandleInstanceId, method: MethodId, data: &[u8]| {
                    dispatcher_clone.dispatch_and_call(handle, method, data)
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

    pub fn call(&self, data: Vec<u8>) -> Vec<u8> {
        self.call(data)
    }
}

impl<D: Dispatcher> Drop for Port<D> {
    fn drop(&mut self) {
        self.server.terminate();
    }
}

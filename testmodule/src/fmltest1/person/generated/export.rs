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

use super::super::get_context;
use super::super::handles;
use fml::handle::pool::HandlePool;
use fml::handle::{ExportedHandle, HandleInstanceId};
use fml::port::PortId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn get_handle_pool(port_id: PortId) -> Arc<ExportedHandles> {
    get_context().ports.lock().unwrap().get(&port_id).unwrap().1.dispatcher_get()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Customer {
    pub handle: ExportedHandle,
}

pub struct ExportedHandles {
    pub port_id: PortId,
    pub handles_trait1: HandlePool<dyn handles::Customer + Send + Sync>,
}

impl ExportedHandles {
    pub fn create_handle_customer<T: handles::Customer + Send + Sync + 'static>(&self, x: T) -> Customer {
        let trait_id = 1 as u16;
        let index = self.handles_trait1.create(Arc::new(x)) as u16;
        Customer {
            handle: ExportedHandle {
                port_id: self.port_id,
                id: HandleInstanceId {
                    trait_id,
                    index,
                },
            },
        }
    }
}

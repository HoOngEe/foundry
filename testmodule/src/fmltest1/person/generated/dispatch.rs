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

use super::super::handles;
use super::export::ExportedHandles;
use crate::fml::handle::Dispatcher;
use crate::fml::handle::{HandleInstanceId, MethodId};
use crate::fml::port::PortId;
use fml::handle::pool::HandlePool;
use fml::PacketHeader;
use std::io::Cursor;
use std::sync::Arc;

fn dispatch_1(
    mut buffer: Cursor<&mut Vec<u8>>,
    object: Arc<dyn handles::Customer + Send + Sync>,
    method: MethodId,
    data: &[u8],
) {
    match method {
        1 => {
            let (a1, a2) = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
            let result = object.add_criminal_record(a1, a2);
            serde_cbor::to_writer(&mut buffer, &result).unwrap();
        }
        2 => {
            let (a1,) = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
            let result = object.reform(a1);
            serde_cbor::to_writer(&mut buffer, &result).unwrap();
        }
        3 => {
            let (a1,) = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
            let result = object.provoke(a1);
            serde_cbor::to_writer(&mut buffer, &result).unwrap();
        }
        _ => panic!("Invalid method id given"),
    }
}

impl Dispatcher for ExportedHandles {
    fn new(port_id: PortId, size: usize) -> Self {
        ExportedHandles {
            port_id,
            handles_trait1: HandlePool::new(size),
        }
    }

    fn dispatch_and_call(&self, buffer: Cursor<&mut Vec<u8>>, handle: HandleInstanceId, method: MethodId, data: &[u8]) {
        match handle.trait_id {
            1 => dispatch_1(buffer, self.handles_trait1.get(handle.index as usize), method, data),
            _ => panic!(),
        }
    }
}

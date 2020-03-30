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
use std::sync::Arc;

fn dispatch_1(buffer: &mut [u8], object: Arc<dyn handles::Bank + Send + Sync>, method: MethodId, data: &[u8]) {
    match method {
        1 => {
            let (a1, a2) = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
            let result = object.deposit(a1, a2);
            serde_json::to_writer(&mut buffer[std::mem::size_of::<PacketHeader>()..], &result).unwrap();
        }
        2 => {
            let (a1, a2) = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
            let result = object.kill_the_clerk(a1, a2);
            serde_json::to_writer(&mut buffer[std::mem::size_of::<PacketHeader>()..], &result).unwrap();
        }
        3 => {
            let (a1,) = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
            let result = object.check_balance(a1);
            serde_json::to_writer(&mut buffer[std::mem::size_of::<PacketHeader>()..], &result).unwrap();
        }
        4 => {
            let () = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
            let result = object.ask_nearest_police_station();
            serde_json::to_writer(&mut buffer[std::mem::size_of::<PacketHeader>()..], &result).unwrap();
        }
        _ => panic!("Invalid method id given"),
    }
}

fn dispatch_2(buffer: &mut [u8], object: Arc<dyn handles::PoliceStation + Send + Sync>, method: MethodId, data: &[u8]) {
    match method {
        1 => {
            let (a1,) = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
            let result = object.turn_yourself_in(a1);
            serde_json::to_writer(&mut buffer[std::mem::size_of::<PacketHeader>()..], &result).unwrap();
        }
        2 => {
            let () = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
            let result = object.kill_the_police();
            serde_json::to_writer(&mut buffer[std::mem::size_of::<PacketHeader>()..], &result).unwrap();
        }
        _ => panic!("Invalid method id given"),
    }
}

impl Dispatcher for ExportedHandles {
    fn new(port_id: PortId, size: usize) -> Self {
        ExportedHandles {
            port_id,
            handles_trait1: HandlePool::new(size),
            handles_trait2: HandlePool::new(size),
        }
    }

    fn dispatch_and_call(&self, buffer: &mut [u8], handle: HandleInstanceId, method: MethodId, data: &[u8]) {
        match handle.trait_id {
            1 => dispatch_1(buffer, self.handles_trait1.get(handle.index as usize), method, data),
            2 => dispatch_2(buffer, self.handles_trait2.get(handle.index as usize), method, data),
            _ => panic!(),
        }
    }
}

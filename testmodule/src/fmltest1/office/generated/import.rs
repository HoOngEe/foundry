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
use super::super::handles::Customer as TCustomer;
use fml::handle::{ImportedHandle, MethodId};
use fml::PacketHeader;

pub struct Customer {
    pub handle: ImportedHandle,
}

fn call<T: serde::Serialize, R: serde::de::DeserializeOwned>(handle: &ImportedHandle, method: MethodId, args: &T) -> R {
    let mut buffer: Vec<u8> = Vec::new();
    buffer.resize(std::mem::size_of::<PacketHeader>(), 0 as u8);
    serde_json::to_writer(&mut buffer[std::mem::size_of::<PacketHeader>()..], &args).unwrap();
    let result = get_context().ports.lock().unwrap().get(&handle.port_id).unwrap().1.call(handle.id, method, buffer);
    serde_cbor::from_reader(&result[std::mem::size_of::<PacketHeader>()..]).unwrap()
}

impl TCustomer for Customer {
    fn add_criminal_record(&self, name: String, record: String) {
        call(&self.handle, 1, &(name, record))
    }

    fn reform(&self, name: String) -> bool {
        call(&self.handle, 1, &(name,))
    }

    fn provoke(&self, name: String) -> Customer {
        let handle: ImportedHandle = call(&self.handle, 3, &(name,));
        Customer {
            handle,
        }
    }
}

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

use super::{get_context, impls, import};
use fml::handle::{ExportedHandle, HandlePreset, ImportedHandle};
use fml::port::PortId;

pub struct Preset {}

impl HandlePreset for Preset {
    fn export(&mut self, port_id: PortId) -> Result<ExportedHandle, String> {
        let port_table = get_context().ports.read().unwrap();
        let (config, port) = port_table.get(&port_id).unwrap();
        if config.kind == "person" {
            let bank = port.dispatcher_get().create_handle_bank(impls::JustBank {
                port_id,
            });
            return Ok(bank.handle)
        }
        Err("Nothing to export to this kind of module".to_owned())
    }

    fn import(&mut self, handle: ImportedHandle) -> Result<(), String> {
        if get_context().ports.read().unwrap().get(&handle.port_id).unwrap().0.kind != "person" {
            panic!("Invalid handle import")
        }
        let customer = &mut get_context().custom.customer.lock().unwrap();
        if customer.is_some() {
            return Err("Handle already imported".to_owned())
        }
        **customer = Some(import::Customer {
            handle,
        });
        Ok(())
    }
}

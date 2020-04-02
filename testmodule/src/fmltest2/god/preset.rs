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

use super::{get_context, impls};
use fml::handle::{ExportedHandle, HandlePreset, ImportedHandle};
use fml::port::PortId;

pub struct Preset {}

impl HandlePreset for Preset {
    fn export(&mut self, port_id: PortId) -> Result<ExportedHandle, String> {
        let port_table = get_context().ports.lock().unwrap();
        let (config, port) = port_table.get(&port_id).unwrap();
        if config.kind == "cleric" {
            let bank = port.dispatcher_get().create_handle_weatherforecast(impls::Zeus {});
            return Ok(bank.handle)
        }
        Err("Nothing to export to this kind of module".to_owned())
    }

    fn import(&mut self, handle: ImportedHandle) -> Result<(), String> {
        Ok(())
    }
}

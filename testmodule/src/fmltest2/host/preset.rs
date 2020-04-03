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
        Err("Nothing to export".to_owned())
    }

    fn import(&mut self, handle: ImportedHandle) -> Result<(), String> {
        let kind = get_context().ports.lock().unwrap().get(&handle.port_id).unwrap().0.kind.clone();
        if kind != "human" {
            panic!("Invalid handle import")
        }
        let weather = &mut get_context().custom.weather.lock().unwrap();
        if weather.is_some() {
            return Err("Handle already imported".to_owned())
        }
        **weather = Some(import::WeatherRequest {
            handle,
        });
        Ok(())
    }
}

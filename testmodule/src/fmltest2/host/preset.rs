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

use super::{get_context, import};
use fml::handle::{ExportedHandle, HandlePreset, ImportedHandle};
use fml::port::PortId;

pub struct Preset {}

impl HandlePreset for Preset {
    fn export(&mut self, _port_id: PortId) -> Result<ExportedHandle, String> {
        Err("Nothing to export".to_owned())
    }

    fn import(&mut self, handle: ImportedHandle) -> Result<(), String> {
        let kind = get_context().ports.read().unwrap().get(&handle.port_id).unwrap().0.kind.clone();
        match (kind.as_str(), handle.port_id) {
            ("human", 1) => {
                let weather_request = &mut get_context().custom.weather.lock().unwrap();
                if weather_request.is_some() {
                    return Err("Handle already imported".to_owned())
                }
                **weather_request = Some(import::WeatherRequest {
                    handle,
                });
            }
            ("human", 3) => {
                let pray_request = &mut get_context().custom.pray.lock().unwrap();
                if pray_request.is_some() {
                    return Err("Handle already imported".to_owned())
                }
                **pray_request = Some(import::PrayRequest {
                    handle,
                })
            }
            ("human", 7) => {
                let talk_to_humans = &mut get_context().custom.talk.write().unwrap();
                if talk_to_humans.is_some() {
                    return Err("Handle already imported".to_owned())
                }
                **talk_to_humans = Some(import::TalkToHumans {
                    handle,
                })
            }
            _ => panic!("Invalid handle import"),
        };
        Ok(())
    }
}

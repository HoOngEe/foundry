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
        match (config.kind.as_str(), port_id) {
            ("cleric", 1) => {
                let weather_request = port.dispatcher_get().create_handle_weatherforecast(impls::Zeus {});
                Ok(weather_request.handle)
            }
            ("cleric", 3) => {
                let rain_oracle_giver = port.dispatcher_get().create_handle_rainoraclegiver(impls::Zeus {});
                Ok(rain_oracle_giver.handle)
            }
            ("cleric", 11) => {
                let talk_to_gods = port.dispatcher_get().create_handle_talktogods(impls::Gaia {
                    power: 5,
                });
                Ok(talk_to_gods.handle)
            }
            ("human", 13) => {
                let talk_to_gods = port.dispatcher_get().create_handle_talktogods(impls::Gaia {
                    power: 5,
                });
                Ok(talk_to_gods.handle)
            }
            _ => Err("Nothing to export to this kind of module".to_owned()),
        }
    }

    fn import(&mut self, handle: ImportedHandle) -> Result<(), String> {
        let kind = get_context().ports.read().unwrap().get(&handle.port_id).unwrap().0.kind.clone();
        match (kind.as_str(), handle.port_id) {
            ("cleric", 10) => {
                let talk_to_clerics = &mut get_context().custom.talk_to_clerics.write().unwrap();
                if talk_to_clerics.is_some() {
                    return Err("Handle already imported".to_owned())
                }
                **talk_to_clerics = Some(import::TalkToClerics {
                    handle,
                })
            }
            ("human", 12) => {
                let talk_to_humans = &mut get_context().custom.talk_to_humans.write().unwrap();
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

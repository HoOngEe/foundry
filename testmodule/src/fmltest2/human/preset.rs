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

use super::{core::types::GroundState, get_context, impls, import};
use fml::handle::{ExportedHandle, HandlePreset, ImportedHandle};
use fml::port::PortId;

pub struct Preset {}

impl HandlePreset for Preset {
    fn export(&mut self, port_id: PortId) -> Result<ExportedHandle, String> {
        let port_table = get_context().ports.read().unwrap();
        let (config, port) = port_table.get(&port_id).unwrap();
        match (config.kind.as_str(), port_id) {
            ("host", 1) => {
                let weather_request = port.dispatcher_get().create_handle_weatherrequest(impls::WeatherForecaster {});
                Ok(weather_request.handle)
            }
            ("host", 3) => {
                let pray_request = port.dispatcher_get().create_handle_prayrequest(impls::Farmer {
                    farm_state: GroundState::Drought,
                });
                Ok(pray_request.handle)
            }
            ("cleric", 6) => {
                let ground_observer = port.dispatcher_get().create_handle_groundobserver(impls::Farmer {
                    farm_state: GroundState::Drought,
                });
                Ok(ground_observer.handle)
            }
            ("host", 7) => {
                let talk_to_humans = port.dispatcher_get().create_handle_talktohumans(impls::OpenMindedCitizen {
                    power: 2,
                });
                Ok(talk_to_humans.handle)
            }
            ("cleric", 9) => {
                let talk_to_humans = port.dispatcher_get().create_handle_talktohumans(impls::OpenMindedCitizen {
                    power: 2,
                });
                Ok(talk_to_humans.handle)
            }
            ("god", 12) => {
                let talk_to_humans = port.dispatcher_get().create_handle_talktohumans(impls::OpenMindedCitizen {
                    power: 2,
                });
                Ok(talk_to_humans.handle)
            }
            _ => Err("Nothing to export to this kind of module".to_owned()),
        }
    }

    fn import(&mut self, handle: ImportedHandle) -> Result<(), String> {
        let kind = get_context().ports.read().unwrap().get(&handle.port_id).unwrap().0.kind.clone();
        match (kind.as_str(), handle.port_id) {
            ("cleric", 2) => {
                let weather_response = &mut get_context().custom.weather_response.lock().unwrap();
                if weather_response.is_some() {
                    return Err("Handle already imported".to_owned())
                }
                **weather_response = Some(import::WeatherResponse {
                    handle,
                });
            }
            ("cleric", 4) => {
                let pray_response = &mut get_context().custom.pray_response.lock().unwrap();
                if pray_response.is_some() {
                    return Err("Handle already imported".to_owned())
                }
                **pray_response = Some(import::PrayResponse {
                    handle,
                })
            }
            ("cleric", 8) => {
                let talk_to_clerics = &mut get_context().custom.talk_to_clerics.write().unwrap();
                if talk_to_clerics.is_some() {
                    return Err("Handle already imported".to_owned())
                }
                **talk_to_clerics = Some(import::TalkToClerics {
                    handle,
                })
            }
            ("god", 13) => {
                let talk_to_gods = &mut get_context().custom.talk_to_gods.write().unwrap();
                if talk_to_gods.is_some() {
                    return Err("Handle already imported".to_owned())
                }
                **talk_to_gods = Some(import::TalkToGods {
                    handle,
                })
            }
            _ => panic!("Invalid handle import"),
        };
        Ok(())
    }
}

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
            ("human", 2) => {
                let weather_response = port.dispatcher_get().create_handle_weatherresponse(impls::Bishop {});
                Ok(weather_response.handle)
            }
            ("human", 4) => {
                let pray_response = port.dispatcher_get().create_handle_prayresponse(impls::Priest {});
                Ok(pray_response.handle)
            }
            ("human", 8) => {
                let pray_response = port.dispatcher_get().create_handle_talktoclerics(impls::Cardinal {
                    power: 3,
                });
                Ok(pray_response.handle)
            }
            ("god", 10) => {
                let pray_response = port.dispatcher_get().create_handle_talktoclerics(impls::Cardinal {
                    power: 3,
                });
                Ok(pray_response.handle)
            }
            _ => Err("Nothing to export to this kind of module".to_owned()),
        }
    }

    fn import(&mut self, handle: ImportedHandle) -> Result<(), String> {
        let kind = get_context().ports.read().unwrap().get(&handle.port_id).unwrap().0.kind.clone();
        match (kind.as_str(), handle.port_id) {
            ("god", 1) => {
                let weather_forecast = &mut get_context().custom.weather_forecast.lock().unwrap();
                if weather_forecast.is_some() {
                    Err("Handle already imported".to_owned())
                } else {
                    Ok(**weather_forecast = Some(import::WeatherForecast {
                        handle,
                    }))
                }
            }
            ("god", 3) => {
                let rain_oracle_giver = &mut get_context().custom.rain_oracle_giver.lock().unwrap();
                if rain_oracle_giver.is_some() {
                    return Err("Handle already imported".to_owned())
                } else {
                    Ok(**rain_oracle_giver = Some(import::RainOracleGiver {
                        handle,
                    }))
                }
            }
            ("human", 6) => {
                let ground_observer = &mut get_context().custom.ground_observer.lock().unwrap();
                if ground_observer.is_some() {
                    return Err("Handle already imported".to_owned())
                } else {
                    Ok(**ground_observer = Some(import::GroundObserver {
                        handle,
                    }))
                }
            }
            ("human", 9) => {
                let talk_to_humans = &mut get_context().custom.talk_to_humans.write().unwrap();
                if talk_to_humans.is_some() {
                    return Err("Handle already imported".to_owned())
                } else {
                    Ok(**talk_to_humans = Some(import::TalkToHumans {
                        handle,
                    }))
                }
            }
            ("god", 11) => {
                let talk_to_gods = &mut get_context().custom.talk_to_gods.write().unwrap();
                if talk_to_gods.is_some() {
                    return Err("Handle already imported".to_owned())
                } else {
                    Ok(**talk_to_gods = Some(import::TalkToGods {
                        handle,
                    }))
                }
            }
            _ => panic!("Invalid handle import"),
        }
    }
}

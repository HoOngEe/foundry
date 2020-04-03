#[cfg(test)]
mod tests {
    #[fml_macro::fml_macro_debug]
    pub mod handles {
        #[exported]
        pub trait WeatherRequest {
            fn weather(&self, date: String) -> Weather;
        }

        #[imported]
        pub trait WeatherResponse {
            fn weather(&self, date: String) -> Weather;
        }
    }

    #[cfg(kungs)]
    fn this_is_just_for_macro_expansion() {
        pub mod handles {
            pub use super::generated::{export, import};
            use super::types::*;
            pub trait WeatherResponse {
                fn weather(&self, date: String) -> Weather;
            }
            pub trait WeatherForecast {
                fn weather(&self, date: String) -> Weather;
            }
        }
        pub mod generated {
            pub mod dispatch {
                use super::super::handles;
                use super::super::types::*;
                use super::export::ExportedHandles;
                use crate::fml::handle::Dispatcher;
                use crate::fml::handle::{HandleInstanceId, MethodId};
                use crate::fml::port::PortId;
                use fml::handle::pool::HandlePool;
                use fml::PacketHeader;
                use std::io::Cursor;
                use std::sync::Arc;
                #[allow(clippy::let_unit_value)]
                fn dispatch_1(
                    mut buffer: Cursor<&mut Vec<u8>>,
                    object: Arc<dyn handles::WeatherResponse + Send + Sync>,
                    method: MethodId,
                    data: &[u8],
                ) {
                    match method {
                        1 => {
                            let (a1,) = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
                            let result = object.weather(a1);
                            serde_cbor::to_writer(&mut buffer, &result).unwrap();
                        }
                        _ => panic!(),
                    }
                }
                impl Dispatcher for ExportedHandles {
                    fn new(port_id: PortId, size: usize) -> Self {
                        ExportedHandles {
                            port_id,
                            handles_trait1: HandlePool::new(size),
                        }
                    }
                    fn dispatch_and_call(
                        &self,
                        buffer: Cursor<&mut Vec<u8>>,
                        handle: HandleInstanceId,
                        method: MethodId,
                        data: &[u8],
                    ) {
                        match handle.trait_id {
                            1 => {
                                dispatch_1(buffer, self.handles_trait1.get(handle.index as usize), method, data);
                            }
                            _ => panic!(),
                        }
                    }
                }
            }
            pub mod export {
                use super::super::super::get_context;
                use super::super::handles;
                use super::super::types::*;
                use fml::handle::pool::HandlePool;
                use fml::handle::{ExportedHandle, HandleInstanceId};
                use fml::port::PortId;
                use serde::{Deserialize, Serialize};
                use std::sync::Arc;
                pub fn get_handle_pool(port_id: PortId) -> Arc<ExportedHandles> {
                    get_context().ports.lock().unwrap().get(&port_id).unwrap().1.dispatcher_get()
                }
                #[derive(Serialize, Deserialize, Debug)]
                pub struct WeatherResponse {
                    pub handle: ExportedHandle,
                }
                pub struct ExportedHandles {
                    pub port_id: PortId,
                    pub handles_trait1: HandlePool<dyn handles::WeatherResponse + Send + Sync>,
                }
                impl ExportedHandles {
                    pub fn create_handle_weatherresponse<T: handles::WeatherResponse + Send + Sync + 'static>(
                        &self,
                        x: T,
                    ) -> WeatherResponse {
                        let trait_id = 1 as u16;
                        let index = self.handles_trait1.create(Arc::new(x)) as u16;
                        WeatherResponse {
                            handle: ExportedHandle {
                                port_id: self.port_id,
                                id: HandleInstanceId {
                                    trait_id,
                                    index,
                                },
                            },
                        }
                    }
                }
            }
            pub mod import {
                use super::super::super::get_context;
                use super::super::handles::WeatherForecast as TWeatherForecast;
                use super::super::types::*;
                use fml::handle::{ImportedHandle, MethodId};
                use fml::PacketHeader;
                use serde::{Deserialize, Serialize};
                pub fn call<T: serde::Serialize, R: serde::de::DeserializeOwned>(
                    handle: &ImportedHandle,
                    method: MethodId,
                    args: &T,
                ) -> R {
                    let mut buffer: Vec<u8> = Vec::new();
                    buffer.resize(std::mem::size_of::<PacketHeader>(), 0 as u8);
                    serde_cbor::to_writer(&mut buffer[std::mem::size_of::<PacketHeader>()..], &args).unwrap();
                    let result = get_context()
                        .ports
                        .lock()
                        .unwrap()
                        .get(&handle.port_id)
                        .unwrap()
                        .1
                        .call(handle.id, method, buffer);
                    serde_cbor::from_reader(&result[std::mem::size_of::<PacketHeader>()..]).unwrap()
                }
                #[derive(Serialize, Deserialize, Debug)]
                pub struct WeatherForecast {
                    pub handle: ImportedHandle,
                }
            }
            pub mod import_impls {
                use super::super::handles::WeatherForecast as TWeatherForecast;
                use super::super::types::*;
                use super::import;
                impl TWeatherForecast for import::WeatherForecast {
                    fn weather(&self, date: String) -> Weather {
                        super::import::call(&self.handle, 1, &(date,))
                    }
                }
            }
        }
    }
}

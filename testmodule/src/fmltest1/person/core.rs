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

/*
HOW TO DEFINE HANDLES

1. import::* must appear only in the return type of a method in the imported handle
2. export::* must appear only in the return type of a method in the exported handle.
3. Both import::* and export::* may appear in a complex type
as far as it's on the right spot (the return type).
For example, the return type is allowed to be Vec<(u8, import::SomeHandle)>
4. You must always use &self for both imported / exported handles. (&mut self it not possible)
If you want mutablity to internal state of Handle or Global context, you should consider Mutex.
*/

pub mod handles {
    pub use super::generated::{export, import};
    pub trait Bank {
        fn deposit(&self, name: String, amount: u64) -> u64;
        fn kill_the_clerk(&self, name: String, weapon: String) -> bool;
        fn check_balance(&self, name: String) -> u64;
        fn ask_nearest_police_station(&self) -> import::PoliceStation;
    }

    pub trait PoliceStation {
        fn turn_yourself_in(&self, bail: u64) -> String;
        fn kill_the_police(&self);
    }

    pub trait Customer {
        fn add_criminal_record(&self, name: String, record: String);
        fn reform(&self, name: String) -> bool;
        fn provoke(&self, name: String) -> export::Customer;
    }
}

pub mod generated {
    pub mod dispatch {
        use super::super::handles;
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
            object: Arc<dyn handles::Customer + Send + Sync>,
            method: MethodId,
            data: &[u8],
        ) {
            match method {
                1 => {
                    let (a1, a2) = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
                    let result = object.add_criminal_record(a1, a2);
                    serde_cbor::to_writer(&mut buffer, &result).unwrap();
                }
                2 => {
                    let (a1,) = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
                    let result = object.reform(a1);
                    serde_cbor::to_writer(&mut buffer, &result).unwrap();
                }
                3 => {
                    let (a1,) = serde_cbor::from_reader(&data[std::mem::size_of::<PacketHeader>()..]).unwrap();
                    let result = object.provoke(a1);
                    serde_cbor::to_writer(&mut buffer, &result).unwrap();
                }
                _ => panic!("Invalid method id given"),
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
                    1 => dispatch_1(buffer, self.handles_trait1.get(handle.index as usize), method, data),
                    _ => panic!(),
                }
            }
        }
    }

    pub mod export {
        use super::super::super::get_context;
        use super::super::handles;
        use fml::handle::pool::HandlePool;
        use fml::handle::{ExportedHandle, HandleInstanceId};
        use fml::port::PortId;
        use serde::{Deserialize, Serialize};
        use std::sync::Arc;

        pub fn get_handle_pool(port_id: PortId) -> Arc<ExportedHandles> {
            get_context().ports.lock().unwrap().get(&port_id).unwrap().1.dispatcher_get()
        }

        #[derive(Serialize, Deserialize, Debug)]
        pub struct Customer {
            pub handle: ExportedHandle,
        }

        pub struct ExportedHandles {
            pub port_id: PortId,
            pub handles_trait1: HandlePool<dyn handles::Customer + Send + Sync>,
        }

        impl ExportedHandles {
            pub fn create_handle_customer<T: handles::Customer + Send + Sync + 'static>(&self, x: T) -> Customer {
                let trait_id = 1 as u16;
                let index = self.handles_trait1.create(Arc::new(x)) as u16;
                Customer {
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
        use super::super::handles::{Bank as TBank, PoliceStation as TPoliceStation};
        use fml::handle::{ImportedHandle, MethodId};
        use fml::PacketHeader;

        pub struct Bank {
            pub handle: ImportedHandle,
        }

        pub struct PoliceStation {
            pub handle: ImportedHandle,
        }

        pub fn call<T: serde::Serialize, R: serde::de::DeserializeOwned>(
            handle: &ImportedHandle,
            method: MethodId,
            args: &T,
        ) -> R {
            let mut buffer: Vec<u8> = Vec::new();
            buffer.resize(std::mem::size_of::<PacketHeader>(), 0 as u8);
            serde_json::to_writer(&mut buffer[std::mem::size_of::<PacketHeader>()..], &args).unwrap();
            let result =
                get_context().ports.lock().unwrap().get(&handle.port_id).unwrap().1.call(handle.id, method, buffer);
            serde_cbor::from_reader(&result[std::mem::size_of::<PacketHeader>()..]).unwrap()
        }

        impl TBank for Bank {
            fn deposit(&self, name: String, amount: u64) -> u64 {
                call(&self.handle, 1, &(name, amount))
            }

            fn kill_the_clerk(&self, name: String, weapon: String) -> bool {
                call(&self.handle, 2, &(name, weapon))
            }

            fn check_balance(&self, name: String) -> u64 {
                call(&self.handle, 3, &(name,))
            }

            fn ask_nearest_police_station(&self) -> PoliceStation {
                let handle: ImportedHandle = call(&self.handle, 3, &());
                PoliceStation {
                    handle,
                }
            }
        }

        impl TPoliceStation for PoliceStation {
            fn turn_yourself_in(&self, bail: u64) -> String {
                call(&self.handle, 1, &(bail,))
            }

            fn kill_the_police(&self) -> () {
                call(&self.handle, 2, &())
            }
        }
    }
}

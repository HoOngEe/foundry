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

pub mod office;
pub mod person;

use cbsb::execution::{executee, executor};
use cbsb::ipc;
use cbsb::ipc::same_process::SameProcess;
use cbsb::ipc::{TwoWayInitializableIpc, TwoWayInitialize};
use fml::context::Config;
use fml::handle::ImportedHandle;
use std::sync::Arc;

const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100000);

pub fn recv<I: ipc::TwoWayInitializableIpc, T: serde::de::DeserializeOwned>(
    ctx: &executor::Context<I, executor::PlainThread>,
) -> T {
    serde_cbor::from_slice(&ctx.ipc.recv(Some(TIMEOUT)).unwrap()).unwrap()
}

pub fn send<I: ipc::TwoWayInitializableIpc, T: serde::Serialize>(
    ctx: &executor::Context<I, executor::PlainThread>,
    data: &T,
) {
    ctx.ipc.send(&serde_cbor::to_vec(data).unwrap());
}

pub fn done_ack<I: ipc::TwoWayInitializableIpc>(ctx: &executor::Context<I, executor::PlainThread>) {
    // Rust doesn't support type ascription yet.
    assert_eq!(
        {
            let x: String = recv(&ctx);
            x
        },
        "done"
    );
}

pub fn run() {
    executor::add_plain_thread_pool("person".to_owned(), Arc::new(|a: Vec<String>| person::main_like_test(a)));
    executor::add_plain_thread_pool("office".to_owned(), Arc::new(|a: Vec<String>| office::main_like_test(a)));

    let ctx1 = executor::execute::<SameProcess, executor::PlainThread>("person").unwrap();
    let ctx2 = executor::execute::<SameProcess, executor::PlainThread>("office").unwrap();

    let config1 = Config {
        kind: "person".to_owned(),
        id: "ID 1".to_owned(),
        key: "Key 1".to_owned(),
        args: b"Arg 1".to_vec(),
    };

    let config2 = Config {
        kind: "office".to_owned(),
        id: "ID 2".to_owned(),
        key: "Key 2".to_owned(),
        args: b"Arg 2".to_vec(),
    };

    let port_id = 1 as usize; // This is the id in its own port list, so 1 for both.

    send(&ctx1, &config1);
    send(&ctx2, &config2);

    let linker = <SameProcess as TwoWayInitializableIpc>::Linker::new("BaseSandbox".to_owned());
    let (ipc_config1, ipc_config2) = linker.create();

    send(&ctx1, &"link");
    send(&ctx1, &(port_id, config2, serde_cbor::to_vec(&"SameProcess").unwrap(), ipc_config1));
    done_ack(&ctx1);

    send(&ctx2, &"link");
    send(&ctx2, &(port_id, config1, serde_cbor::to_vec(&"SameProcess").unwrap(), ipc_config2));
    done_ack(&ctx2);

    send(&ctx1, &"handle_export");
    send(&ctx1, &(port_id,));
    let handle_from_1: ImportedHandle = recv(&ctx1);
    done_ack(&ctx1);

    send(&ctx2, &"handle_export");
    send(&ctx2, &(port_id,));
    let handle_from_2: ImportedHandle = recv(&ctx2);
    done_ack(&ctx2);

    send(&ctx1, &"handle_import");
    send(&ctx1, &(handle_from_2,));
    done_ack(&ctx1);

    send(&ctx2, &"handle_import");
    send(&ctx2, &(handle_from_1,));
    done_ack(&ctx2);

    send(&ctx1, &"call");
    send(&ctx1, &(handle_from_2, 2 as u32, serde_cbor::to_vec(&("John", "Gun")).unwrap()));
    let buffer: Vec<u8> = recv(&ctx1);
    let result: bool = serde_cbor::from_slice(&buffer).unwrap();
    assert_eq!(result, true);

    send(&ctx1, &"terminate");
    send(&ctx2, &"terminate");

    ctx1.terminate();
    ctx2.terminate();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmltest1_simple() {
        run();
    }
}

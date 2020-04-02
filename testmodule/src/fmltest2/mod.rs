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

mod cleric;
mod god;
mod human;

use cbsb::execution::executor;
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

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub enum Weather {
    Sunny,
    Windy,
    Foggy,
    Cloudy,
    Snowy,
    Rainy,
}

pub fn run() {
    let create_context_with_name = |path: &str|  {
        executor::execute::<SameProcess, executor::PlainThread>(path).unwrap()
    };

    executor::add_plain_thread_pool("human".to_owned(), Arc::new(human::main_like_test));
    executor::add_plain_thread_pool("cleric".to_owned(), Arc::new(cleric::main_like_test));
    executor::add_plain_thread_pool("god".to_owned(), Arc::new(god::main_like_test));

    let ctx_human = create_context_with_name("human");
    let ctx_cleric = create_context_with_name("cleric");
    let ctx_god = create_context_with_name("god");

    let config_human = Config {
        kind: "human".to_owned(),
        id: "ID 1".to_owned(),
        key: "Key 1".to_owned(),
        args: b"Arg 1".to_vec(),
    };

    let config_cleric = Config {
        kind: "cleric".to_owned(),
        id: "ID 2".to_owned(),
        key: "Key 2".to_owned(),
        args: b"Arg 2".to_vec(),
    };

    let config_god = Config {
        kind: "god".to_owned(),
        id: "ID 3".to_owned(),
        key: "Key 3".to_owned(),
        args: b"Arg 3".to_vec(),
    };

    let port_id_1 = 1 as usize; // This is the id in its own port list, so 1 for both.
    let port_id_2 = 2 as usize;

    send(&ctx_human, &config_human);
    send(&ctx_cleric, &config_cleric);
    send(&ctx_god, &config_god);

    let linker1 = <SameProcess as TwoWayInitializableIpc>::Linker::new("BaseSandbox".to_owned());
    let linker2 = <SameProcess as TwoWayInitializableIpc>::Linker::new("BaseSandbox".to_owned());
    let (ipc_config_human_cleric, ipc_config_cleric_human) = linker1.create();
    let (ipc_config_cleric_god, ipc_config_god_cleric) = linker2.create();

    send(&ctx_human, &"link");
    send(&ctx_human, &(port_id_1, config_cleric.clone(), serde_cbor::to_vec(&"SameProcess").unwrap(), ipc_config_human_cleric));
    done_ack(&ctx_human);

    send(&ctx_cleric, &"link");
    send(&ctx_cleric, &(port_id_1, config_human, serde_cbor::to_vec(&"SameProcess").unwrap(), ipc_config_cleric_human));
    done_ack(&ctx_cleric);

    send(&ctx_cleric, &"link");
    send(&ctx_cleric, &(port_id_2, config_god, serde_cbor::to_vec(&"SameProcess").unwrap(), ipc_config_cleric_god));
    done_ack(&ctx_cleric);

    send(&ctx_god, &"link");
    send(&ctx_god, &(port_id_2, config_cleric, serde_cbor::to_vec(&"SameProcess").unwrap(), ipc_config_god_cleric));
    done_ack(&ctx_god);

    send(&ctx_human, &"handle_export");
    send(&ctx_human, &(port_id_1,));
    let handle_from_human: ImportedHandle = recv(&ctx_human);
    done_ack(&ctx_human);

    send(&ctx_cleric, &"handle_export");
    send(&ctx_cleric, &(port_id_1,));
    let handle_from_cleric: ImportedHandle = recv(&ctx_cleric);
    done_ack(&ctx_cleric);

    send(&ctx_god, &"handle_export");
    send(&ctx_god, &(port_id_2,));
    let handle_from_god: ImportedHandle = recv(&ctx_god);
    done_ack(&ctx_god);

    send(&ctx_human, &"handle_import");
    send(&ctx_human, &(handle_from_cleric,));
    done_ack(&ctx_human);

    send(&ctx_cleric, &"handle_import");
    send(&ctx_cleric, &(handle_from_god,));
    done_ack(&ctx_cleric);

    send(&ctx_human, &"call");
    send(&ctx_human, &(handle_from_human, 1 as u32, serde_cbor::to_vec(&("A",)).unwrap()));
    let buffer: Vec<u8> = recv(&ctx_human);
    let result: Weather = serde_cbor::from_slice(&buffer).unwrap();
    assert_eq!(result, Weather::Cloudy);
    done_ack(&ctx_human);

    send(&ctx_human, &"terminate");
    send(&ctx_cleric, &"terminate");
    send(&ctx_god, &"terminate");

    ctx_human.terminate();
    ctx_cleric.terminate();
    ctx_god.terminate();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmltest2_simple() {
        run();
    }
}

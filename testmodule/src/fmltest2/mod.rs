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
mod host;
mod human;

use cbsb::execution::executor::{add_plain_thread_pool, execute, Context, PlainThread, ThreadAsProcesss};
use cbsb::ipc;
use cbsb::ipc::same_process::{SameProcess, SameProcessLinker};
use cbsb::ipc::{TwoWayInitializableIpc, TwoWayInitialize};
use fml::context::Config;
use fml::handle::ImportedHandle;
use std::sync::Arc;

const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100000);

pub fn recv<I: ipc::TwoWayInitializableIpc, T: serde::de::DeserializeOwned>(ctx: &Context<I, PlainThread>) -> T {
    serde_cbor::from_slice(&ctx.ipc.recv(Some(TIMEOUT)).unwrap()).unwrap()
}

pub fn send<I: ipc::TwoWayInitializableIpc, T: serde::Serialize>(ctx: &Context<I, PlainThread>, data: &T) {
    ctx.ipc.send(&serde_cbor::to_vec(data).unwrap());
}

pub fn done_ack<I: ipc::TwoWayInitializableIpc>(ctx: &Context<I, PlainThread>) {
    // Rust doesn't support type ascription yet.
    assert_eq!(
        {
            let x: String = recv(&ctx);
            x
        },
        "done"
    );
}

type TestContext = Context<SameProcess, PlainThread>;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub enum Weather {
    Sunny,
    Windy,
    Foggy,
    Cloudy,
    Snowy,
    Rainy,
}

#[inline]
fn create_config_with_module_name(module_name: &str) -> Config {
    Config {
        kind: String::from(module_name),
        id: String::from("ID") + module_name,
        key: String::from("KEY") + module_name,
        args: (String::from("ARG") + module_name).into_bytes(),
    }
}

#[inline]
fn create_context_with_name(path: &str) -> TestContext {
    execute::<SameProcess, PlainThread>(path).unwrap()
}

#[inline]
fn string_with_prefix(prefix: &str, target: &str) -> String {
    String::from(prefix) + target
}

fn initialize_module(module_name: &str, module_process: ThreadAsProcesss) -> (TestContext, Config) {
    add_plain_thread_pool(String::from(module_name), module_process);
    let ctx_module = create_context_with_name(module_name);
    let config_module = create_config_with_module_name(module_name);
    send(&ctx_module, &config_module);
    (ctx_module, config_module)
}

macro_rules! create_ctx_and_config {
    (($ctx_name:ident, $ctx_config:ident, $module_name:literal, $process:expr)) => {
        let ($ctx_name, $ctx_config) = initialize_module($module_name, Arc::new($process));
    };
    // recursion
    ($head:tt, $($tail:tt),* $(,)?) => {
        create_ctx_and_config!($head);
        $(
            create_ctx_and_config!($tail);
        )*
    };
}

#[inline]
fn create_same_process_linker() -> SameProcessLinker {
    <SameProcess as TwoWayInitializableIpc>::Linker::new(String::from("BaseSandbox"))
}

fn link_two_modules_through_port(
    module1: (&TestContext, &Config),
    module2: (&TestContext, &Config),
    port_id: usize,
    end_points: (Vec<u8>, Vec<u8>),
) {
    let (ipc_endpoint1, ipc_endpoint2) = end_points;
    let (ctx1, config1) = module1;
    let (ctx2, config2) = module2;
    send(ctx1, &"link");
    send(ctx1, &(port_id, config2.clone(), serde_cbor::to_vec(&"SameProcess").unwrap(), ipc_endpoint1));
    done_ack(ctx1);

    send(ctx2, &"link");
    send(ctx2, &(port_id, config1.clone(), serde_cbor::to_vec(&"SameProcess").unwrap(), ipc_endpoint2));
    done_ack(ctx2);
}

fn export_handle_from_module_port(ctx: &TestContext, port_id: usize) -> ImportedHandle {
    send(ctx, &"handle_export");
    send(ctx, &(port_id,));
    let handle: ImportedHandle = recv(ctx);
    done_ack(ctx);
    handle
}

fn import_handle_to_module(ctx: &TestContext, handle: ImportedHandle) {
    send(ctx, &"handle_import");
    send(ctx, &(handle,));
    done_ack(ctx);
}

fn terminate_modules(ctxs: Vec<TestContext>) {
    ctxs.into_iter().for_each(|ctx| {
        send(&ctx, &"terminate");
        ctx.terminate();
    });
}

fn call_module_function<'a, T: serde::de::DeserializeOwned>(
    ctx: &TestContext,
    handle: ImportedHandle,
    method_idx: u32,
    args: Vec<u8>,
) -> T {
    send(&ctx, &"call");
    send(&ctx, &(handle, method_idx, args));
    let buffer: Vec<u8> = recv(&ctx);
    let result: T = serde_cbor::from_slice(&buffer).unwrap();
    done_ack(&ctx);
    result
}

pub fn run_for_weather() {
    create_ctx_and_config!(
        (ctx_human, config_human, "human", human::main_like_test),
        (ctx_host, config_host, "host", host::main_like_test),
        (ctx_cleric, config_cleric, "cleric", cleric::main_like_test),
        (ctx_god, config_god, "god", god::main_like_test)
    );

    // linkers should live long
    let linker_host_human = create_same_process_linker();
    let linker_human_cleric = create_same_process_linker();
    let linker_cleric_god = create_same_process_linker();

    link_two_modules_through_port(
        (&ctx_host, &config_host),
        (&ctx_human, &config_human),
        1,
        linker_host_human.create(),
    );
    link_two_modules_through_port(
        (&ctx_human, &config_human),
        (&ctx_cleric, &config_cleric),
        2,
        linker_human_cleric.create(),
    );
    link_two_modules_through_port(
        (&ctx_cleric, &config_cleric),
        (&ctx_god, &config_god),
        1,
        linker_cleric_god.create(),
    );

    let weather_request_handle = export_handle_from_module_port(&ctx_human, 1);
    let weather_response_handle = export_handle_from_module_port(&ctx_cleric, 2);
    let weather_forecast_handle = export_handle_from_module_port(&ctx_god, 1);

    import_handle_to_module(&ctx_host, weather_request_handle);
    import_handle_to_module(&ctx_human, weather_response_handle);
    import_handle_to_module(&ctx_cleric, weather_forecast_handle);

    let weather_request_arg = serde_cbor::to_vec(&("A",)).unwrap();
    let weather: Weather = call_module_function(&ctx_host, weather_request_handle, 1, weather_request_arg);
    assert_eq!(weather, Weather::Rainy);

    terminate_modules(vec![ctx_human, ctx_host, ctx_cleric, ctx_god]);
}

pub fn run_for_pray() {
    create_ctx_and_config!(
        (ctx_human, config_human, "human", human::main_like_test),
        (ctx_host, config_host, "host", host::main_like_test),
        (ctx_cleric, config_cleric, "cleric", cleric::main_like_test),
        (ctx_god, config_god, "god", god::main_like_test)
    );

    // linkers should live long
    let linker_host_human = create_same_process_linker();
    let linker_human_cleric = create_same_process_linker();
    let linker_human_cleric2 = create_same_process_linker();
    let linker_cleric_god = create_same_process_linker();

    link_two_modules_through_port(
        (&ctx_host, &config_host),
        (&ctx_human, &config_human),
        3,
        linker_host_human.create(),
    );
    link_two_modules_through_port(
        (&ctx_human, &config_human),
        (&ctx_cleric, &config_cleric),
        4,
        linker_human_cleric.create(),
    );
    link_two_modules_through_port(
        (&ctx_human, &config_human),
        (&ctx_cleric, &config_cleric),
        6,
        linker_human_cleric2.create(),
    );
    link_two_modules_through_port(
        (&ctx_cleric, &config_cleric),
        (&ctx_god, &config_god),
        3,
        linker_cleric_god.create(),
    );

    let pray_request_handle = export_handle_from_module_port(&ctx_human, 3);
    let ground_observer_handle = export_handle_from_module_port(&ctx_human, 6);
    let pray_response_handle = export_handle_from_module_port(&ctx_cleric, 4);
    let rain_oracle_giver_handle = export_handle_from_module_port(&ctx_god, 3);

    import_handle_to_module(&ctx_host, pray_request_handle);
    import_handle_to_module(&ctx_human, pray_response_handle);
    import_handle_to_module(&ctx_cleric, rain_oracle_giver_handle);
    import_handle_to_module(&ctx_cleric, ground_observer_handle);

    let pray_request_arg = serde_cbor::to_vec(&()).unwrap();
    let admiration: String = call_module_function(&ctx_host, pray_request_handle, 1, pray_request_arg);
    assert_eq!(admiration, "Your majesty, my farm will fournish thanks for your Heavy rain");

    terminate_modules(vec![ctx_human, ctx_host, ctx_cleric, ctx_god]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmltest2_simple() {
        run_for_weather();
    }

    #[test]
    fn fmltest2_pray() {
        run_for_pray();
    }
}

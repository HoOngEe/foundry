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

#[cfg(test)]
mod fmltest2 {
    use cbsb::execution::executor::{add_plain_thread_pool, execute, Context, PlainThread, ThreadAsProcesss};
    use cbsb::ipc;
    use cbsb::ipc::same_process::{SameProcess, SameProcessLinker};
    use cbsb::ipc::{TwoWayInitializableIpc, TwoWayInitialize};
    use fml::context::Config;
    use fml::handle::ImportedHandle;
    use std::sync::Arc;

    use super::*;
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

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Issue {
        Entangled(String, u64),
        Resolved,
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

    pub fn run_for_talk() {
        create_ctx_and_config!(
            (ctx_human, config_human, "human", human::main_like_test),
            (ctx_host, config_host, "host", host::main_like_test),
            (ctx_cleric, config_cleric, "cleric", cleric::main_like_test),
            (ctx_god, config_god, "god", god::main_like_test)
        );

        // linkers should live long
        let linker_host_human_talk_to_humans = create_same_process_linker();
        let linker_human_cleric_talk_to_clerics = create_same_process_linker();
        let linker_human_cleric_talk_to_humans = create_same_process_linker();
        let linker_cleric_god_talk_to_clerics = create_same_process_linker();
        let linker_cleric_god_talk_to_gods = create_same_process_linker();
        let linker_human_god_talk_to_humans = create_same_process_linker();
        let linker_human_god_talk_to_gods = create_same_process_linker();

        link_two_modules_through_port(
            (&ctx_host, &config_host),
            (&ctx_human, &config_human),
            7,
            linker_host_human_talk_to_humans.create(),
        );
        link_two_modules_through_port(
            (&ctx_cleric, &config_cleric),
            (&ctx_human, &config_human),
            8,
            linker_human_cleric_talk_to_clerics.create(),
        );
        link_two_modules_through_port(
            (&ctx_cleric, &config_cleric),
            (&ctx_human, &config_human),
            9,
            linker_human_cleric_talk_to_humans.create(),
        );
        link_two_modules_through_port(
            (&ctx_cleric, &config_cleric),
            (&ctx_god, &config_god),
            10,
            linker_cleric_god_talk_to_clerics.create(),
        );
        link_two_modules_through_port(
            (&ctx_cleric, &config_cleric),
            (&ctx_god, &config_god),
            11,
            linker_cleric_god_talk_to_gods.create(),
        );
        link_two_modules_through_port(
            (&ctx_human, &config_human),
            (&ctx_god, &config_god),
            12,
            linker_human_god_talk_to_humans.create(),
        );
        link_two_modules_through_port(
            (&ctx_human, &config_human),
            (&ctx_god, &config_god),
            13,
            linker_human_god_talk_to_gods.create(),
        );

        let talk_to_humans_for_host = export_handle_from_module_port(&ctx_human, 7);
        let talk_to_humans_for_cleric = export_handle_from_module_port(&ctx_human, 9);
        let talk_to_humans_for_god = export_handle_from_module_port(&ctx_human, 12);

        let talk_to_clerics_for_human = export_handle_from_module_port(&ctx_cleric, 8);
        let talk_to_clerics_for_god = export_handle_from_module_port(&ctx_cleric, 10);

        let talk_to_gods_for_human = export_handle_from_module_port(&ctx_god, 13);
        let talk_to_gods_for_cleric = export_handle_from_module_port(&ctx_god, 11);

        import_handle_to_module(&ctx_host, talk_to_humans_for_host);

        import_handle_to_module(&ctx_human, talk_to_clerics_for_human);
        import_handle_to_module(&ctx_human, talk_to_gods_for_human);

        import_handle_to_module(&ctx_cleric, talk_to_humans_for_cleric);
        import_handle_to_module(&ctx_cleric, talk_to_gods_for_cleric);

        import_handle_to_module(&ctx_god, talk_to_humans_for_god);
        import_handle_to_module(&ctx_god, talk_to_clerics_for_god);

        [2u64, 5, 20, 199, 7182, 19892900].iter().for_each(|degree| {
            let issue = vec![Issue::Entangled(String::from("Host"), *degree)];
            let pray_request_arg = serde_cbor::to_vec(&(issue.clone(),)).unwrap();
            let result: Vec<Issue> = call_module_function(&ctx_host, talk_to_humans_for_host, 1, pray_request_arg);
            assert_eq!(result, talk_result_oracle(issue));
        });
        terminate_modules(vec![ctx_human, ctx_host, ctx_cleric, ctx_god]);
    }

    fn talk_result_oracle(mut issue: Vec<Issue>) -> Vec<Issue> {
        let positions = ["Human", "Cleric", "God"];
        let rotator = |prev, degree| {
            let current = if degree % 2 == 0 {
                let mut iter = positions.iter().cycle().skip_while(|position| **position != prev);
                iter.next();
                iter.next().unwrap()
            } else {
                let mut iter = positions.iter().rev().cycle().skip_while(|posititon| **posititon != prev);
                iter.next();
                iter.next().unwrap()
            };
            String::from(*current)
        };

        match issue.last() {
            Some(Issue::Entangled(prev_loc, degree)) if *degree > 0 => {
                let next_loc = match (prev_loc.as_str(), degree) {
                    ("Host", _) => String::from("Human"),
                    (prev, degree) => rotator(prev, degree),
                };
                let decreased_degree = match next_loc.as_str() {
                    "Host" => degree / 2,
                    "Human" => degree / 2,
                    "Cleric" => degree / 3,
                    "God" => degree / 5,
                    _ => unreachable!(),
                };
                issue.push(Issue::Entangled(next_loc, decreased_degree));
                talk_result_oracle(issue)
            }
            _ => {
                issue.push(Issue::Resolved);
                issue
            }
        }
    }

    #[test]
    fn fmltest2_simple() {
        run_for_weather();
    }

    #[test]
    fn fmltest2_pray() {
        run_for_pray();
    }

    #[test]
    fn fmltest3_talk() {
        run_for_talk();
    }
}

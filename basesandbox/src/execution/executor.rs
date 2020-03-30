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

use crate::ipc::*;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

pub trait Executor {
    fn new(path: &str, args: &[&str]) -> Self;
    fn join(&mut self);
}

pub struct Executable {
    child: std::process::Child,
}

impl Executor for Executable {
    fn new(path: &str, args: &[&str]) -> Self {
        let mut command = Command::new(path);
        command.args(args);
        Executable {
            child: command.spawn().unwrap(),
        }
    }

    fn join(&mut self) {
        // This is synchronized with excutee's signal (#TERMINATE),
        // which is supposed to be sent in termination step.
        // Thus, in normal case, it won't take much time to be in a waitable status.
        // However a malicous excutee might send a signal arbitrarily before the termination.
        // For that, we have a timeout for the wait.
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            if self.child.try_wait().unwrap().is_some() {
                return // successful
            }
        }
        panic!("Module hasn't terminated itself. Protocol Error")
    }
}

lazy_static! {
    static ref POOL: Mutex<HashMap<String, Box<dyn Fn(Vec<String>) -> () + Send + Sync>>> =
        { Mutex::new(HashMap::new()) };
}

pub fn add_plain_thread_pool(key: String, f: Box<dyn Fn(Vec<String>) -> () + Send + Sync>) {
    POOL.lock().unwrap().insert(key, f);
}

pub struct PlainThread {
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Executor for PlainThread {
    fn new(path: &str, args: &[&str]) -> Self {
        let path = path.to_owned();
        let args: Vec<String> = args.iter().map(|x| x.to_string()).collect();
        let handle = std::thread::spawn(move || POOL.lock().unwrap().get(&path).unwrap()(args));

        PlainThread {
            handle: Some(handle),
        }
    }

    fn join(&mut self) {
        // PlainThread Executor is for test, so no worry for malicous unresponsiveness
        self.handle.take().unwrap().join().unwrap();
    }
}

/// Rust doesn't allow Drop for trait, so we need this
/// See E0120
struct ExecutorWrapper<T: Executor> {
    executor: T,
}

impl<T: Executor> Drop for ExecutorWrapper<T> {
    fn drop(&mut self) {
        self.executor.join();
    }
}

/// declaration order of fields is important because of Drop dependencies
pub struct Context<T: TwoWayInitializableIpc, E: Executor> {
    pub ipc: T,
    _child: ExecutorWrapper<E>,
    _linker: T::Linker,
}

/// id must be unique for each instance.
pub fn execute<T: TwoWayInitializableIpc, E: Executor>(path: &str) -> Result<Context<T, E>, String> {
    let linker = T::Linker::new("BaseSandbox".to_owned());
    let (config_server, config_client) = linker.create();
    let ipc = T::new(config_server);
    let config_client = hex::encode(&config_client);
    let args: Vec<&str> = vec![&config_client];
    let child = ExecutorWrapper {
        executor: Executor::new(path, &args),
    };
    let ping = ipc.recv(Some(Duration::from_millis(100))).unwrap();
    assert_eq!(ping, b"#INIT\0");
    Ok(Context {
        ipc,
        _child: child,
        _linker: linker,
    })
}

/// Call this when you're sure that the excutee is ready to teminate; i.e.
/// it will call excutee::terminate() asap.
pub fn terminate<T: TwoWayInitializableIpc, E: Executor>(ctx: Context<T, E>) {
    let ping = ctx.ipc.recv(Some(Duration::from_millis(50))).unwrap();
    assert_eq!(ping, b"#TERMINATE\0");
}

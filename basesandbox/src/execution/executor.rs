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

use crate::ipc::domain_socket::DomainSocket;
use crate::ipc::*;
use std::process::Command;
use std::time::Duration;

const TEMPORARY_PATH: &str = "./tmp/";

struct Executor {
    child: std::process::Child,
}

impl Executor {
    fn new(path: &str, args: &[&str]) -> Self {
        let mut command = Command::new(path);
        command.args(args);
        Executor {
            child: command.spawn().unwrap(),
        }
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
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

struct DirectoryReserver {
    path: String,
}

impl DirectoryReserver {
    fn new(path: String) -> Self {
        std::fs::create_dir(&path).unwrap();
        DirectoryReserver {
            path,
        }
    }
}

impl Drop for DirectoryReserver {
    fn drop(&mut self) {
        std::fs::remove_dir(&self.path).unwrap();
    }
}

/// declaration order of fields is important because of Drop dependencies
pub struct Context<T: Ipc> {
    pub ipc: T,
    _child: Executor,
    _directory: DirectoryReserver,
}

/// id must be unique for each instance.
pub fn execute<T: Ipc + TwoWayInitialize>(path: &str) -> Result<Context<T>, String> {
    std::fs::remove_dir_all("./tmp").ok(); // we don't care whether it succeeds
    let directory = DirectoryReserver::new("./tmp".to_owned());
    let (ipc_config, ipc_config_next) = T::create(init_data_from_path(TEMPORARY_PATH.to_owned()));
    let ipc = T::new(ipc_config);
    let ipc_config_next = hex::encode(&ipc_config_next);
    let args: Vec<&str> = vec![&ipc_config_next];
    let child = Executor::new(path, &args);
    let ping = ipc.recv(Some(Duration::from_millis(100))).unwrap();
    assert_eq!(ping, b"#INIT\0");

    Ok(Context {
        ipc,
        _child: child,
        _directory: directory,
    })
}

/// Call this when you're sure that the excutee is ready to teminate; i.e.
/// it will call excutee::terminate() asap.
pub fn terminate<T: Ipc>(ctx: Context<T>) {
    let ping = ctx.ipc.recv(Some(Duration::from_millis(50))).unwrap();
    assert_eq!(ping, b"#TERMINATE\0");
}

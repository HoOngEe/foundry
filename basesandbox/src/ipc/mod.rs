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

pub mod domain_socket;
pub mod semaphore;
use std::collections::hash_map::HashMap;

/// Inter-process obejct that is constructed / synchronized by another Inter-process mechanism.
pub trait InterProcessUnit {
    /// Constructs itself with an opaque data that would have been transported by some IPC
    fn new(data: Vec<u8>) -> Self;
    /// All other inter-process-ly linked objects are ready. You're ready to start your work.
    fn ready(&mut self);
}

/// Some implementors of InterProcessUnit would take a single 'path'
/// since it's quite a trivial model that inter-process objects try to acquire an access to a
/// inter-process handle that is identified by a system-wide name.
pub fn init_data_from_path(path: String) -> Vec<u8> {
    let mut config: HashMap<String, String> = HashMap::new();
    config.insert("path".to_owned(), path);
    serde_cbor::to_vec(&config).unwrap()
}

/// Abstraction of IPC communication
pub trait IpcSend: Send {
    /// It might block until counterparty's recv(). Even if not, the order is still guaranteed.
    fn send(&self, data: &[u8]);
}

#[derive(Debug, PartialEq)]
pub enum RecvFlag {
    TimeOut,
    Termination,
}

pub trait Terminate: Send {
    /// Wake up block on recv with a special flag
    fn terminate(&self);
}

pub trait IpcRecv: Send {
    /// Returns Err only for the timeout or termination wake-up(otherwise panic)
    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvFlag>;
    /// Create a terminate switch that can be sent to another thread
    fn create_terminate(&self) -> Box<dyn Terminate>;
}

pub trait Ipc: InterProcessUnit + IpcSend + IpcRecv {
    // Some implementor of Ipc might provide split(), which consumes itself and return pair of IpcSend and IpcRecv
}

/// Most of IPC depends on a system-wide-name, which looks quite vulnerable for
/// possible attack. Rather, generating a random name would be more secure.
pub fn generate_random_name() -> String {
    let pid = std::process::id();
    let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    let hash = ccrypto::blake256(format!("{:?}{}", time, pid));
    format!("{:?}", hash)[0..16].to_string()
}

pub trait TwoWayInitialize: InterProcessUnit {
    /// From an opaque configuration, creates two configurations,
    /// which will be feeded to InterProcessUnit::new(),
    /// for the server(caller) itself and the following client (another prcoess).
    fn create(config: Vec<u8>) -> (Vec<u8>, Vec<u8>);
}

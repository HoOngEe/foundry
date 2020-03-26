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

use crate::port::Port;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::handle::Dispatcher;

pub struct Config {
    /// kind of this module. Per-binary
    pub kind: String,
    /// id of this instance of module. Per-instance, Per-appdescriptor
    pub id: String,
    /// key of this instance of module. Per-instance, Per-execution
    pub key: String,
    /// Arguments given to this module.
    pub args: Vec<u8>,
}

/// You can add additional variable as you want.
/// However, be careful of doing so, since it will be hihgly likely to cause a nondeterministic behavior.
/// Try to keep stateless as possible, and custmoize this only for the cache purpose.
pub trait Custom {
    fn new(context: &Config) -> Self;
}

/// A global context that will be accessible from this module
pub struct Context<T : Custom, D: Dispatcher> {
    /// Internal objects
    ports: Arc<Mutex<HashMap<String, Port<D>>>>,

    /// Meta, pre-decided constant variables
    pub config: Config,

    /// Custom variables
    pub custom: T,
}

impl<T : Custom, D: Dispatcher> Context<T, D> {
    pub fn new(ports: Arc<Mutex<HashMap<String, Port<D>>>>, config: Config, custom: T) -> Self {
        Context {
            ports,
            config,
            custom
        }
    }
}

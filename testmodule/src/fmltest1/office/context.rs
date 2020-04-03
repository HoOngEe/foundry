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

pub use super::core::generated::*;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct Context {
    pub customer: Mutex<Option<import::Customer>>,
    pub accounts: Mutex<HashMap<String, u64>>,
}

impl fml::context::Custom for Context {
    fn new(_context: &fml::context::Config) -> Self {
        Context {
            customer: Mutex::new(None),
            accounts: Mutex::new(HashMap::new()),
        }
    }
}
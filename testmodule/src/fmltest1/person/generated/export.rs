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

use fml::handle::pool::HandlePool;
use fml::handle::Dispatch;
use fml::handle::{ExportedHandle};
use super::super::handles::{Customer as TCustomer};

pub struct Customer {
    handle: ExportedHandle
}

trait TTCustomer: Dispatch + Send + Sync {
    fn add_criminal_record(&mut self, name: &str, record: &str);
    fn reform(&self, name: &str) -> bool;
    fn provoke(&self, name: &str) -> Customer;
}

//macro will automatically generate this
pub struct ExportedHandles {
    pub handles_trait1: HandlePool<dyn TCustomer>,
}

impl ExportedHandles {
    pub fn new(size: usize) -> Self {
        ExportedHandles {
            handles_trait1: HandlePool::new(size),
        }
    }
}

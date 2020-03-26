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

use super::pool::HandlePool;
use super::Dispatch;

pub trait Trait1: Dispatch + Send + Sync {
    fn hi(&self, a: i32) -> f32;
}

pub trait Trait2: Dispatch + Send + Sync {
    fn bye(&self, a: i32, b: String) -> f32;
}

//macro will automatically generate this
pub struct ExportedHandles {
    pub handles_trait1: HandlePool<dyn Trait1>,
    pub handles_trait2: HandlePool<dyn Trait2>,
}

impl ExportedHandles {
    pub fn new(size: usize) -> Self {
        ExportedHandles {
            handles_trait1: HandlePool::new(size),
            handles_trait2: HandlePool::new(size),
        }
    }
}

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

use super::{HandleInstanceId, MethodId};

impl super::export::ExportedHandles {
    pub fn dispatch_and_call(&self, handle: HandleInstanceId, method: MethodId, data: &[u8]) -> Vec<u8> {
        let handle_type = (handle << 16) >> 16;
        let handle_index = handle >> 16;

        match handle_type {
            1 => self.handles_trait1.get(handle_index as usize).dispatch(method, data),
            2 => self.handles_trait2.get(handle_index as usize).dispatch(method, data),
            _ => panic!(),
        }
    }
}

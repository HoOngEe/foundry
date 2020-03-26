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

pub mod pool;

/// 16 less significant bits: trait of the handle & 16 more significant bits: index in a pool
pub type HandleInstanceId = u32;
pub type MethodId = u32;

pub type TraitId = u32;
pub type StructId = u32;

pub struct ExportedHandle {
    trait_id: TraitId,
    struct_id: StructId,
}

pub struct ImportedHandle {
    trait_id: TraitId,
    port_id: String, // TODO: improve this with an integer index.
}

/// This will be implemented per module
pub trait Dispatcher : Send + Sync {
    fn dispatch_and_call(&self, handle: HandleInstanceId, method: MethodId, data: &[u8]) -> Vec<u8>;
}

/// This will be implemented per trait
pub trait Dispatch {
    fn dispatch(&self, method_id: MethodId, data: &[u8]) -> Vec<u8>;
}

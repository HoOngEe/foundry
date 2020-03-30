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

use super::port::PortId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HandleInstanceId {
    pub trait_id: u16,
    pub index: u16,
}

pub type MethodId = u32;

pub type TraitId = u32;
pub type StructId = u32;

#[derive(Serialize, Deserialize, Debug)]
pub struct ExportedHandle {
    pub id: HandleInstanceId,
    pub port_id: PortId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImportedHandle {
    pub id: HandleInstanceId,
    pub port_id: PortId,
}

/// This will be implemented per module, but instantiated per link.
pub trait Dispatcher: Send + Sync {
    fn new(port_id: PortId, size: usize) -> Self;
    fn dispatch_and_call(&self, buffer: &mut [u8], handle: HandleInstanceId, method: MethodId, data: &[u8]);
}

/// This will be implemented per trait (of Handle)
pub trait Dispatch {
    fn dispatch(&self, method_id: MethodId, data: &[u8]) -> Vec<u8>;
}

// Default, preset handler provider
pub trait HandlePreset {
    fn export(&mut self, port_id: PortId) -> Result<ExportedHandle, String>;
    fn import(&mut self, handle: ImportedHandle) -> Result<(), String>;
}

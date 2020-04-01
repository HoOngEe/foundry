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

pub mod context;
pub mod core;
pub mod descriptor;
mod impls;
pub mod preset;

pub use self::core::generated::*;
use cbsb::ipc::domain_socket::DomainSocket;
use cbsb::ipc::same_process::SameProcess;
use context::Context as MyContext;
use preset::Preset;

pub type Context = fml::context::Context<MyContext, export::ExportedHandles>;
static mut CONTEXT: Option<Context> = None;

pub fn get_context() -> &'static Context {
    unsafe { CONTEXT.as_ref().unwrap() }
}

pub fn main_like(args: Vec<String>) {
    let mut preset = Preset {};
    fml::core::<DomainSocket, MyContext, export::ExportedHandles, Preset>(
        args,
        &mut preset,
        Box::new(|ctx: Context| unsafe {
            CONTEXT.replace(ctx);
        }),
    );
}

pub fn main_like_test(args: Vec<String>) {
    let mut preset = Preset {};
    fml::core::<SameProcess, MyContext, export::ExportedHandles, Preset>(
        args,
        &mut preset,
        Box::new(|ctx: Context| unsafe {
            CONTEXT.replace(ctx);
        }),
    );
}

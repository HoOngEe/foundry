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

pub mod office;
pub mod person;

use cbsb::execution::{executee, executor};
use cbsb::ipc;

const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);

pub fn recv<I: ipc::TwoWayInitializableIpc, T: serde::de::DeserializeOwned>(
    ctx: &executor::Context<I, executor::PlainThread>,
) -> T {
    serde_cbor::from_slice(&ctx.ipc.recv(Some(TIMEOUT)).unwrap()).unwrap()
}

pub fn send<I: ipc::TwoWayInitializableIpc, T: serde::Serialize>(
    ctx: &executor::Context<I, executor::PlainThread>,
    data: &T,
) {
    ctx.ipc.send(&serde_cbor::to_vec(data).unwrap());
}

pub fn run() {
    executor::add_plain_thread_pool("person".to_owned(), Box::new(|a: Vec<String>| person::main_like_test(a)));
    executor::add_plain_thread_pool("office".to_owned(), Box::new(|a: Vec<String>| office::main_like_test(a)));

    let ctx1 = executor::execute::<ipc::domain_socket::DomainSocket, executor::PlainThread>("person").unwrap();
    let ctx2 = executor::execute::<ipc::domain_socket::DomainSocket, executor::PlainThread>("office").unwrap();

    send(&ctx1, &"Kind 1".to_owned());
    send(&ctx1, &"Id 1".to_owned());
    send(&ctx1, &"Key 1".to_owned());
    send(&ctx1, b"Arg 1");

    send(&ctx2, &"Kind 2".to_owned());
    send(&ctx2, &"Id 2".to_owned());
    send(&ctx2, &"Key 2".to_owned());
    send(&ctx2, b"Arg 2");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmltest1_simple() {}
}

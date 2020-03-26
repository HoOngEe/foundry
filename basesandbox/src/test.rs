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

#[cfg(test)]
mod tests {
    use crate::execution::executor;
    use crate::ipc::domain_socket::DomainSocket;
    use crate::ipc::{IpcRecv, IpcSend};
    use std::time::Duration;

    #[cfg(target_os = "linux")]
    #[test]
    fn simple_rust() {
        let ctx = executor::execute::<DomainSocket>("./../target/debug/tm_simple_rs").unwrap();

        ctx.ipc.send(b"Hello?\0");
        let r = ctx.ipc.recv(Some(Duration::from_millis(100))).unwrap();
        assert_eq!(r, b"I'm here!\0");
    }
}

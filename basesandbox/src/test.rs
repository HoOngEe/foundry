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
    use crate::directory::{DirectoryReserver, TEMPORARY_PATH};
    use crate::execution::executee;
    use crate::execution::executor;
    use crate::ipc::domain_socket::DomainSocket;
    use crate::ipc::same_process::SameProcess;
    use crate::ipc::TwoWayInitializableIpc;
    use crate::ipc::{IpcRecv, IpcSend};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    fn simple_thread(args: Vec<String>) {
        let ctx = executee::start::<SameProcess>(args);
        let r = ctx.ipc.as_ref().unwrap().recv(Some(Duration::from_millis(100))).unwrap();
        assert_eq!(r, b"Hello?\0");
        ctx.ipc.as_ref().unwrap().send(b"I'm here!\0");
        ctx.terminate();
    }

    fn simple_executor<I: TwoWayInitializableIpc, E: executor::Executor>(path: &str) {
        let ctx = executor::execute::<I, E>(path).unwrap();
        ctx.ipc.send(b"Hello?\0");
        let r = ctx.ipc.recv(Some(Duration::from_millis(100))).unwrap();
        assert_eq!(r, b"I'm here!\0");
        ctx.terminate();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn simple_rust() {
        let _dr = DirectoryReserver::new(TEMPORARY_PATH.to_owned());
        simple_executor::<DomainSocket, executor::Executable>("./../target/debug/tm_simple_rs");
    }

    #[test]
    fn simple_inprocess() {
        executor::add_plain_thread_pool("simple_thread".to_owned(), Arc::new(|a: Vec<String>| simple_thread(a)));
        simple_executor::<SameProcess, executor::PlainThread>("simple_thread");
    }

    #[test]
    fn multiple() {
        let _dr = DirectoryReserver::new(TEMPORARY_PATH.to_owned());
        executor::add_plain_thread_pool("simple_thread1".to_owned(), Arc::new(|a: Vec<String>| simple_thread(a)));

        let t1 =
            thread::spawn(|| simple_executor::<DomainSocket, executor::Executable>("./../target/debug/tm_simple_rs"));
        let t2 =
            thread::spawn(|| simple_executor::<DomainSocket, executor::Executable>("./../target/debug/tm_simple_rs"));
        let t3 =
            thread::spawn(|| simple_executor::<DomainSocket, executor::Executable>("./../target/debug/tm_simple_rs"));
        let t4 = thread::spawn(|| simple_executor::<SameProcess, executor::PlainThread>("simple_thread1"));
        let t5 = thread::spawn(|| simple_executor::<SameProcess, executor::PlainThread>("simple_thread1"));
        let t6 = thread::spawn(|| simple_executor::<SameProcess, executor::PlainThread>("simple_thread1"));

        t1.join().unwrap();
        t2.join().unwrap();
        t3.join().unwrap();
        t4.join().unwrap();
        t5.join().unwrap();
        t6.join().unwrap();
    }

    #[test]
    fn complicated() {
        executor::add_plain_thread_pool("simple_thread1".to_owned(), Arc::new(|a: Vec<String>| simple_thread(a)));
        let ctx1 = executor::execute::<SameProcess, executor::PlainThread>("simple_thread1").unwrap();
        let ctx2 = executor::execute::<SameProcess, executor::PlainThread>("simple_thread1").unwrap();

        ctx1.ipc.send(b"Hello?\0");
        let r = ctx1.ipc.recv(Some(Duration::from_millis(100))).unwrap();
        assert_eq!(r, b"I'm here!\0");

        ctx2.ipc.send(b"Hello?\0");
        let r = ctx2.ipc.recv(Some(Duration::from_millis(100))).unwrap();
        assert_eq!(r, b"I'm here!\0");

        ctx1.terminate();
        ctx2.terminate();
    }
}

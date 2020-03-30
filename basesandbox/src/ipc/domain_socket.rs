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

use super::*;
use std::cell::RefCell;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::sync::Arc;

const TEMPORARY_PATH: &str = "./tmp";

struct DirectoryReserver {
    path: String,
}

impl DirectoryReserver {
    fn new(path: String) -> Self {
        std::fs::create_dir(&path).unwrap();
        DirectoryReserver {
            path,
        }
    }
}

impl Drop for DirectoryReserver {
    fn drop(&mut self) {
        std::fs::remove_dir(&self.path).unwrap();
    }
}

pub struct DomainSocketLinker {
    address_server: String,
    address_client: String,
    _directory: DirectoryReserver,
}

impl TwoWayInitialize for DomainSocketLinker {
    type Server = DomainSocket;
    type Client = DomainSocket;

    fn new(_name: String) -> Self {
        std::fs::remove_dir_all(TEMPORARY_PATH).ok(); // we don't care whether it succeeds
        let directory = DirectoryReserver::new(TEMPORARY_PATH.to_owned());

        let address_server = format!("{}/{}", TEMPORARY_PATH, generate_random_name());
        let address_client = format!("{}/{}", TEMPORARY_PATH, generate_random_name());
        DomainSocketLinker {
            address_server,
            address_client,
            _directory: directory,
        }
    }

    fn create(&self) -> (Vec<u8>, Vec<u8>) {
        (
            serde_cbor::to_vec(&(&self.address_server, &self.address_client)).unwrap(),
            serde_cbor::to_vec(&(&self.address_client, &self.address_server)).unwrap(),
        )
    }
}

impl Drop for DomainSocketLinker {
    fn drop(&mut self) {
        std::fs::remove_file(&self.address_server).unwrap();
        std::fs::remove_file(&self.address_client).unwrap();
    }
}

struct SocketInternal(UnixDatagram);
impl Drop for SocketInternal {
    fn drop(&mut self) {
        self.0.shutdown(std::net::Shutdown::Both).unwrap();
    }
}

pub struct DomainSocketSend {
    address_src: String,
    address_dst: String,
    socket: Arc<SocketInternal>,
}

impl IpcSend for DomainSocketSend {
    fn send(&self, data: &[u8]) {
        assert_eq!(self.socket.0.send_to(data, &self.address_dst).unwrap(), data.len());
    }
}

pub struct DomainSocketRecv {
    address_src: String,
    address_dst: String,
    socket: Arc<SocketInternal>,
    buffer: RefCell<Vec<u8>>,
}

impl IpcRecv for DomainSocketRecv {
    type MyTerminate = Terminator;

    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvFlag> {
        self.socket.0.set_read_timeout(timeout).unwrap();
        let (count, address) = self.socket.0.recv_from(&mut self.buffer.borrow_mut()).unwrap();
        assert!(count <= self.buffer.borrow().len(), "Unix datagram got data larger than the buffer.");
        if count == 0 {
            return Err(RecvFlag::Termination)
        }
        assert_eq!(
            address.as_pathname().unwrap(),
            Path::new(&self.address_dst),
            "Unix datagram received packet from an unexpected sender."
        );
        Ok(self.buffer.borrow()[0..count].to_vec())
    }

    fn create_terminate(&self) -> Self::MyTerminate {
        Terminator(self.socket.clone())
    }
}

pub struct Terminator(Arc<SocketInternal>);

impl Terminate for Terminator {
    fn terminate(&self) {
        (self.0).0.shutdown(std::net::Shutdown::Both).unwrap();
    }
}

pub struct DomainSocket {
    send: DomainSocketSend,
    recv: DomainSocketRecv,
}

impl InterProcessUnit for DomainSocket {
    fn new(data: Vec<u8>) -> Self {
        let (address_src, address_dst): (String, String) = serde_cbor::from_slice(&data).unwrap();
        let socket = Arc::new(SocketInternal(UnixDatagram::bind(&address_src).unwrap()));
        DomainSocket {
            send: DomainSocketSend {
                address_src: address_src.clone(),
                address_dst: address_dst.clone(),
                socket: socket.clone(),
            },
            recv: DomainSocketRecv {
                address_src,
                address_dst,
                socket,
                buffer: RefCell::new(vec![0; 1024]),
            },
        }
    }
    fn ready(&mut self) {}
}

impl IpcSend for DomainSocket {
    fn send(&self, data: &[u8]) {
        self.send.send(data)
    }
}

impl IpcRecv for DomainSocket {
    type MyTerminate = Terminator;

    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvFlag> {
        self.recv.recv(timeout)
    }

    fn create_terminate(&self) -> Self::MyTerminate {
        self.recv.create_terminate()
    }
}

impl Ipc for DomainSocket {
    type SendOnly = DomainSocketSend;
    type RecvOnly = DomainSocketRecv;

    fn split(self) -> (Self::SendOnly, Self::RecvOnly) {
        (self.send, self.recv)
    }
}

impl TwoWayInitializableIpc for DomainSocket {
    type Linker = DomainSocketLinker;
}

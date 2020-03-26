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
use std::collections::hash_map::HashMap;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct DomainSocket {
    address_src: String,
    address_dst: String,
    socket: Arc<UnixDatagram>,
    buffer: Mutex<Vec<u8>>,
}

impl Drop for DomainSocket {
    fn drop(&mut self) {
        self.socket.shutdown(std::net::Shutdown::Both).unwrap();
        std::fs::remove_file(&self.address_src).unwrap();
    }
}

impl InterProcessUnit for DomainSocket {
    fn new(data: Vec<u8>) -> Self {
        let (address_src, address_dst) = serde_cbor::from_slice(&data).unwrap();
        let socket = UnixDatagram::bind(&address_src).unwrap();
        DomainSocket {
            address_src,
            address_dst,
            socket: Arc::new(socket),
            buffer: Mutex::new(vec![0; 1024]),
        }
    }

    fn ready(&mut self) {}
}

impl DomainSocket {
    pub fn create_sender(&self) -> DomainSocketSendOnly {
        DomainSocketSendOnly {
            address_dst: self.address_dst.clone(),
            socket: self.socket.clone(),
        }
    }
}

impl IpcSend for DomainSocket {
    fn send(&self, data: &[u8]) {
        assert_eq!(self.socket.send_to(data, &self.address_dst).unwrap(), data.len());
    }
}

pub struct Terminator(Arc<UnixDatagram>);

impl Terminate for Terminator {
    fn terminate(&self) {
        self.0.shutdown(std::net::Shutdown::Both).unwrap();
    }
}

impl IpcRecv for DomainSocket {
    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvFlag> {
        let mut buffer = self.buffer.lock().unwrap();
        self.socket.set_read_timeout(timeout).unwrap();
        let (count, address) = self.socket.recv_from(&mut buffer).unwrap();
        assert!(count <= buffer.len(), "Unix datagram got data larger than the buffer.");
        if count == 0 {
            return Err(RecvFlag::Termination)
        }
        assert_eq!(
            address.as_pathname().unwrap(),
            Path::new(&self.address_dst),
            "Unix datagram received packet from an unexpected sender."
        );
        Ok(buffer[0..count].to_vec())
    }

    fn create_terminate(&self) -> Box<dyn Terminate> {
        Box::new(Terminator(self.socket.clone()))
    }
}

// Having multiple senders is quite natural for channel-like model.
pub struct DomainSocketSendOnly {
    address_dst: String,
    socket: Arc<UnixDatagram>,
}

impl IpcSend for DomainSocketSendOnly {
    fn send(&self, data: &[u8]) {
        assert_eq!(self.socket.send_to(data, &self.address_dst).unwrap(), data.len());
    }
}

impl Ipc for DomainSocket {}

impl TwoWayInitialize for DomainSocket {
    fn create(config: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
        let config: HashMap<String, String> = serde_cbor::from_slice(&config).unwrap();
        let path = config.get("path").unwrap();

        let address_server = format!("{}{}", path, generate_random_name());
        let address_client = format!("{}{}", path, generate_random_name());

        (
            serde_cbor::to_vec(&(address_server.clone(), address_client.clone())).unwrap(),
            serde_cbor::to_vec(&(address_client, address_server)).unwrap(),
        )
    }
}

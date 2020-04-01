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

/*
use super::*;
use nix::errno::Errno;
use nix::libc;
use std::collections::hash_map::HashMap;

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    CREATE,
    OPEN,
}

/// Inter-process semaphore using POSIX
pub struct Semaphore {
    raw: *mut libc::sem_t,
    mode: Mode,
    name: String,
}

fn create(name: &str, mode: Mode) -> *mut libc::sem_t {
    unsafe {
        if Mode::CREATE == mode {
            libc::sem_unlink(name.as_ptr() as *const i8);
        }

        let semaphore = match mode {
            Mode::CREATE => libc::sem_open(
                name.as_ptr() as *const i8,
                nix::fcntl::OFlag::O_CREAT.bits(),
                (libc::S_IRWXU | libc::S_IRWXG | libc::S_IRWXO) as libc::c_uint,
                0,
            ),
            Mode::OPEN => libc::sem_open(name.as_ptr() as *const i8, 0),
        };
        assert_ne!(semaphore, libc::SEM_FAILED, "Failed to create semaphore: {}", Errno::last());
        semaphore
    }
}

impl InterProcessUnit for Semaphore {
    fn new(data: Vec<u8>) -> Self {
        let (address, mode): (String, u8) = serde_cbor::from_slice(&data).unwrap();
        let mode = if mode == 1 {
            Mode::CREATE
        } else if mode == 2 {
            Mode::OPEN
        } else {
            panic!("Invalid Semaphore argument")
        };

        Semaphore {
            raw: create(&address, mode),
            mode,
            name: address,
        }
    }

    fn ready(&mut self) {}
}

impl Semaphore {
    pub fn wait(&mut self) {
        unsafe {
            libc::sem_wait(self.raw);
        }
    }

    pub fn signal(&mut self) {
        unsafe {
            libc::sem_post(self.raw);
        }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            if let Mode::CREATE = self.mode {
                libc::sem_close(self.raw);
                libc::sem_unlink(self.name.as_ptr() as *const i8);
            }
        }
    }
}

impl TwoWayInitialize for Semaphore {

    fn create(config: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
        let config: HashMap<String, String> = serde_cbor::from_slice(&config).unwrap();
        let path = config.get("path").unwrap();
        let address = format!("{}{}", path, generate_random_name());

        (serde_cbor::to_vec(&(address.clone(), 1)).unwrap(), serde_cbor::to_vec(&(address, 2)).unwrap())
    }
}
*/

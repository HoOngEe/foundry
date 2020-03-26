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

use crate::queue::Queue;
use std::sync::{Arc, Mutex};

const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(10);

// Per-trait pool
pub struct HandlePool<T: ?Sized> {
    handles: Arc<Mutex<Vec<Option<Arc<T>>>>>,
    token: Arc<Queue<usize>>,
}

impl<T: ?Sized> HandlePool<T> {
    pub fn new(size: usize) -> Self {
        let pool = HandlePool {
            handles: Arc::new(Mutex::new(Vec::new())),
            token: Arc::new(Queue::new(size)),
        };

        {
            let mut handles = pool.handles.lock().unwrap();
            for i in 0..size {
                handles.push(None);
                pool.token.push(i);
            }
        }
        pool
    }

    pub fn create(&self, x: Arc<T>) -> usize {
        let token = self.token.pop(Some(TIMEOUT)).expect("Too many handle creation reqests");
        let mut handles = self.handles.lock().unwrap();
        assert!((*handles)[token].is_none(), "HandlePool corrupted");
        (*handles)[token] = Some(x);
        token
    }

    pub fn remove(&self, token: usize) {
        let mut handles = self.handles.lock().unwrap();
        assert!((*handles)[token].is_some(), "HandlePool corrupted");
        (*handles)[token].take();
        self.token.push(token);
    }

    pub fn get(&self, token: usize) -> Arc<T> {
        let handles = self.handles.lock().unwrap();
        (*handles)[token].as_ref().unwrap().clone()
    }
}

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

mod generated;
pub mod handles;

pub use generated::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type Context = Arc<fml::context::Context<MyContext, export::ExportedHandles>>;

struct MyContext {
    customers: HashMap<String, (u64, Vec<String>)>,
    bank: Option<import::Bank>,
}

impl fml::context::Custom for MyContext {
    fn new(_context: &fml::context::Config) -> Self {
        MyContext {
            customers: HashMap::new(),
            bank: None
        }
    }
}

struct JustCustomer {
    ctx: Context,
}

impl handles::Customer for JustCustomer {
    fn add_criminal_record(&mut self, name: &str, record: &str) {
        self.ctx.custom.customers.get_mut(name).unwrap().1.push(record.to_string());
    }

    fn reform(&self, name: &str) -> bool {
        true
    }

    fn provoke(&self, name: &str) -> export::Customer {

    }
}

struct DangerousCustomer {
    ctx: Context,
    psychopath: bool
}

impl handles::Customer for DangerousCustomer {
    fn add_criminal_record(&mut self, name: &str, record: &str) {
        self.ctx.custom.customers.get_mut(name).unwrap().1.push(record.to_string());
    }

    fn reform(&self, name: &str) -> bool {
        if self.psychopath {
            // Reforming a psychopath is of course impossible, and even will trigger him to kill someone!
            self.ctx.custom.bank.as_ref().unwrap().ask_nearest_police_station().kill_the_police();
            return false
        }
        if self.ctx.custom.customers.get(name).unwrap().0 == 0 {
            // He is so poort that he just refuses the be reformed.
            return false
        }
        true
    }

    fn provoke(&self, name: &str) -> export::Customer {

    }
}

pub fn main_like() {
    let handles = export::ExportedHandles::new(128);
    fml::core::<MyContext, export::ExportedHandles>(handles);
}

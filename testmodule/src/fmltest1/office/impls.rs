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

use super::handles::*;
use super::{export, get_context};
use fml::port::PortId;

pub struct JustBank {
    pub port_id: PortId,
}

impl Bank for JustBank {
    fn deposit(&self, name: String, amount: u64) -> u64 {
        let mut map = get_context().custom.accounts.lock().unwrap();
        let balance = map.get_mut(&name).unwrap();
        *balance += amount;
        *balance
    }

    fn kill_the_clerk(&self, name: String, weapon: String) -> bool {
        println!("{} killed the clerk, using {}!", name, weapon);
        if weapon == "Gun" {
            true
        } else if weapon == "Knife" {
            false
        } else {
            panic!();
        }
    }

    fn check_balance(&self, name: String) -> u64 {
        *get_context().custom.accounts.lock().unwrap().get(&name).unwrap()
    }

    fn ask_nearest_police_station(&self) -> export::PoliceStation {
        export::get_handle_pool(self.port_id).create_handle_police_station(JustPoliceStation {
            port_id: self.port_id,
        })
    }
}

pub struct JustPoliceStation {
    pub port_id: PortId,
}

impl PoliceStation for JustPoliceStation {
    fn turn_yourself_in(&self, bail: u64) -> String {
        "Whatever".to_owned()
    }

    fn kill_the_police(&self) -> () {
        println!("You killed the police!");
    }
}

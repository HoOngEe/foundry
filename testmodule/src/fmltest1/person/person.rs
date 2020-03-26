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

extern crate codechain_fml as fml;

trait Bank {
    fn deposit(&self, name: &str, amount: u64) -> u64;
    fn kill_the_clerk(&self, name: &str, weapon: &str) -> bool;
    fn check_balance(&self, name: &str) -> u64;
    fn ask_nearest_police_station(&self) -> import::PoliceStation;
}

trait PoliceStation {
    fn turn_yourself_in(&self, bail: u64) -> String;
    fn kill_the_police(&self) -> ();
}

trait Customer {
    fn add_criminal_record(&self, name: &str, record: &str);
    fn reform(&self, name: &str) -> bool;
}

struct TrivialCustomer {
    ctx: GlobalContext,
    psychopath: bool
}

impl Customer for TrivialCustomer {
    fn add_criminal_record(&self, name: &str, record: &str) {

    }

    fn reform(&self, name: &str) -> bool {
        if self.psychopath {
            return false
        }
        //ctx.custom.
        true
    }
}

pub fn main_like() {
    fml::core();
}

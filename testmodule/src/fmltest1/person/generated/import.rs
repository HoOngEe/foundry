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

use fml::handle::{ImportedHandle};
use fml::port::Port;
use super::super::handles::{Bank as TBank, PoliceStation as TPoliceStation};
use super::super::Context;

pub struct Bank {

    handle: ImportedHandle
}

impl TBank for Bank {
    fn deposit(&self, name: &str, amount: u64) -> u64 {
    }

    fn kill_the_clerk(&self, name: &str, weapon: &str) -> bool {

    }

    fn check_balance(&self, name: &str) -> u64 {

    }

    fn ask_nearest_police_station(&self) -> PoliceStation {

    }
}

pub struct PoliceStation {
    handle: ImportedHandle
}

impl TPoliceStation for PoliceStation {
    fn turn_yourself_in(&self, bail: u64) -> String {

    }
    fn kill_the_police(&self) -> () {

    }
}

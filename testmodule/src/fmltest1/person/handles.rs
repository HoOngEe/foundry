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

pub use super::generated::*;

/*
HOW TO DEFINE HANDLES

1. import::* must appear only in the return type of a method in the imported handle
2. export::* must appear only in the return type of a method in the exported handle.
3. Both import::* and export::* may appear in a complex type
as far as it's on the right spot (the return type).
For example, the return type is allowed to be Vec<(u8, import::SomeHandle)>
4. You must always use &self for both imported / exported handles. (&mut self it not possible)
If you want mutablity to internal state of Handle or Global context, you should consider Mutex.

*/

pub trait Bank {
    fn deposit(&self, name: &str, amount: u64) -> u64;
    fn kill_the_clerk(&self, name: &str, weapon: &str) -> bool;
    fn check_balance(&self, name: &str) -> u64;
    fn ask_nearest_police_station(&self) -> import::PoliceStation;
}

pub trait PoliceStation {
    fn turn_yourself_in(&self, bail: u64) -> String;
    fn kill_the_police(&self) -> ();
}

pub trait Customer {
    fn add_criminal_record(&mut self, name: &str, record: &str);
    fn reform(&self, name: &str) -> bool;
    fn provoke(&self, name: &str) -> export::Customer;
}

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

pub use super::core::generated::*;
use std::sync::{Mutex, RwLock};

#[derive(Default)]
pub struct Context {
    pub weather_response: Mutex<Option<import::WeatherResponse>>,
    pub pray_response: Mutex<Option<import::PrayResponse>>,
    pub talk_to_clerics: RwLock<Option<import::TalkToClerics>>,
    pub talk_to_gods: RwLock<Option<import::TalkToGods>>,
}

impl fml::context::Custom for Context {
    fn new(_context: &fml::context::Config) -> Self {
        Default::default()
    }
}

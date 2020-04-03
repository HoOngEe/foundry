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

use super::core::{handles::*, types::Weather};

pub struct Zeus {}

impl WeatherForecast for Zeus {
    fn weather(&self, date: String) -> Weather {
        let dice_value = date.into_bytes().into_iter().fold(0, |acc, val| acc + val) % 6;
        match dice_value {
            0 => Weather::Sunny,
            1 => Weather::Windy,
            2 => Weather::Foggy,
            3 => Weather::Cloudy,
            4 => Weather::Snowy,
            5 => Weather::Rainy,
            _ => panic!("Range is restricted"),
        }
    }
}

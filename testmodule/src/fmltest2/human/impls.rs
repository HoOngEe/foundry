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

use super::{
    core::{
        handles::*,
        types::{GroundState, Weather},
    },
    get_context,
};

pub struct WeatherForecaster {}
pub struct Farmer {
    pub farm_state: GroundState,
}

impl WeatherRequest for WeatherForecaster {
    fn weather(&self, date: String) -> Weather {
        get_context().custom.weather_response.lock().unwrap().as_ref().unwrap().weather(date)
    }
}

impl PrayRequest for Farmer {
    fn pray_for_rain(&self) -> String {
        let rain = get_context().custom.pray_response.lock().unwrap().as_ref().unwrap().respond_to_rain_pray();
        match rain {
            Some(rain) => format!("Your majesty, my farm will fournish thanks for your {:?} rain", rain),
            None => format!("Your majesty, my farm is still wet thanks for your help"),
        }
    }
}

impl GroundObserver for Farmer {
    fn submit_ground_state(&self) -> GroundState {
        self.farm_state
    }
}

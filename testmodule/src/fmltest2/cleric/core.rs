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

#[derive(Serialize, Deserialize)]
pub enum Weather {
    Sunny,
    Windy,
    Foggy,
    Cloudy,
    Snowy,
    Rainy,
}

#[fml_macro::fml_macro]
pub mod handles {
    #[exported]
    pub trait WeatherResponse {
        fn weather(&self, date: String) -> super::Weather;
    }

    #[imported]
    pub trait WeatherForecast {
        fn weather(&self, date: String) -> crate::fmltest2::cleric::core::Weather;
    }
}

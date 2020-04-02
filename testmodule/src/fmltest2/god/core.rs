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

pub mod types {
    #[derive(Serialize, Deserialize)]
    pub enum Weather {
        Sunny,
        Windy,
        Foggy,
        Cloudy,
        Snowy,
        Rainy,
    }

    #[derive(Serialize, Deserialize)]
    pub enum GroundState {
        Wet,
        Dry,
        Drought,
    }

    #[derive(Serialize, Deserialize)]
    pub enum Rain {
        Fine,
        Heavy,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub enum Issue {
        Entangled(String, u64),
        Resolved,
    }
}

#[fml_macro::fml_macro]
pub mod handles {
    #[exported]
    pub trait WeatherForecast {
        fn weather(&self, date: String) -> Weather;
    }

    #[exported]
    pub trait RainOracle {
        fn determine_rain_level(&self, ground_state: GroundState) -> Option<Rain>;
    }

    #[exported]
    pub trait RainOracleGiver {
        fn get_rain_oracle(&self) -> export::RainOracle;
    }

    #[exported]
    pub trait TalkToGods {
        fn talk(&self, issue: Vec<Issue>) -> Vec<Issue>;
    }

    #[imported]
    pub trait TalkToClerics {
        fn talk(&self, issue: Vec<Issue>) -> Vec<Issue>;
    }

    #[imported]
    pub trait TalkToHumans {
        fn talk(&self, issue: Vec<Issue>) -> Vec<Issue>;
    }
}

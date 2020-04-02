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
    pub enum Mind {
        Entangled(String, u64),
        Resolved,
    }
}

#[fml_macro::fml_macro]
pub mod handles {
    #[exported]
    pub trait WeatherResponse {
        fn weather(&self, date: String) -> Weather;
    }

    #[imported]
    pub trait WeatherForecast {
        fn weather(&self, date: String) -> Weather;
    }

    #[imported]
    pub trait GroundObserver {
        fn submit_ground_state(&self) -> GroundState;
    }

    #[exported]
    pub trait PrayResponse {
        fn respond_to_rain_pray(&self) -> Option<Rain>;
    }

    #[imported]
    pub trait RainOracle {
        fn determine_rain_level(&self, ground_state: GroundState) -> Option<Rain>;
    }

    #[imported]
    pub trait RainOracleGiver {
        fn get_rain_oracle(&self) -> import::RainOracle;
    }

    #[exported]
    pub trait TalkToClerics {
        fn talk(&self, mind: Vec<Mind>) -> Vec<Mind>;
    }

    #[imported]
    pub trait TalkToHumans {
        fn talk(&self, mind: Vec<Mind>) -> Vec<Mind>;
    }

    #[imported]
    pub trait TalkToGods {
        fn talk(&self, mind: Vec<Mind>) -> Vec<Mind>;
    }
}

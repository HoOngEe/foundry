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
        types::{GroundState, Issue, Rain, Weather},
    },
    get_context,
};

pub struct Zeus {}
pub struct Demeter {}

pub struct Gaia {
    pub power: u64,
}

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

impl RainOracleGiver for Zeus {
    fn get_rain_oracle(&self) -> export::RainOracle {
        export::get_handle_pool(3).create_handle_rainoracle(Demeter {})
    }
}

impl RainOracle for Demeter {
    fn determine_rain_level(&self, ground_state: GroundState) -> Option<Rain> {
        match ground_state {
            GroundState::Wet => None,
            GroundState::Dry => Some(Rain::Fine),
            GroundState::Drought => Some(Rain::Heavy),
        }
    }
}

impl TalkToGods for Gaia {
    fn talk(&self, issue: Vec<Issue>) -> Vec<Issue> {
        let next_mind_state = match issue.last() {
            Some(Issue::Entangled(_, degree)) if *degree > 0 => {
                Issue::Entangled(String::from("God"), degree / self.power)
            }
            _ => Issue::Resolved,
        };
        let mut new_mind = issue;
        match &next_mind_state {
            Issue::Entangled(_, degree) if degree % 2 == 0 => {
                new_mind.push(next_mind_state);
                get_context().custom.talk_to_humans.read().unwrap().as_ref().unwrap().talk(new_mind)
            }
            Issue::Entangled(_, degree) if degree % 2 == 1 => {
                new_mind.push(next_mind_state);
                get_context().custom.talk_to_clerics.read().unwrap().as_ref().unwrap().talk(new_mind)
            }
            _ => {
                new_mind.push(Issue::Resolved);
                new_mind
            }
        }
    }
}

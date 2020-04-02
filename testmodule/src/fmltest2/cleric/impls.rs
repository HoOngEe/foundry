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
        types::{Mind, Rain, Weather},
    },
    get_context,
};

pub struct Bishop {}
pub struct Priest {}
pub struct Cardinal {
    pub power: u64,
}

impl WeatherResponse for Bishop {
    fn weather(&self, date: String) -> Weather {
        get_context().custom.weather_forecast.lock().unwrap().as_ref().unwrap().weather(date)
    }
}

impl PrayResponse for Priest {
    fn respond_to_rain_pray(&self) -> Option<Rain> {
        let rain_oracle = get_context().custom.rain_oracle_giver.lock().unwrap().as_ref().unwrap().get_rain_oracle();
        let ground_state = get_context().custom.ground_observer.lock().unwrap().as_ref().unwrap().submit_ground_state();
        rain_oracle.determine_rain_level(ground_state)
    }
}

impl TalkToClerics for Cardinal {
    fn talk(&self, mind: Vec<Mind>) -> Vec<Mind> {
        let next_mind_state = match mind.last() {
            Some(Mind::Entangled(_, degree)) if *degree > 0 => {
                Mind::Entangled(String::from("Cleric"), degree / self.power)
            }
            _ => Mind::Resolved,
        };
        let mut new_mind = mind;
        match &next_mind_state {
            Mind::Entangled(_, degree) if degree % 2 == 0 => {
                new_mind.push(next_mind_state);
                get_context().custom.talk_to_gods.read().unwrap().as_ref().unwrap().talk(new_mind)
            }
            Mind::Entangled(_, degree) if degree % 2 == 1 => {
                new_mind.push(next_mind_state);
                get_context().custom.talk_to_humans.read().unwrap().as_ref().unwrap().talk(new_mind)
            }
            _ => {
                new_mind.push(Mind::Resolved);
                new_mind
            }
        }
    }
}

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

use super::handles::*;
use super::{export, get_context, Context, MyContext};
use fml::handle::pool::NOT_DECIDED_INDEX;
use fml::handle::{ExportedHandle, HandleInstanceId};
use fml::port::PortId;
use std::collections::HashMap;

struct JustCustomer {
    port_id: PortId,
}

impl Customer for JustCustomer {
    fn add_criminal_record(&self, name: String, record: String) {
        get_context().custom.customers.lock().unwrap().get_mut(&name).unwrap().1.push(record);
    }

    fn reform(&self, name: String) -> bool {
        true
    }

    fn provoke(&self, name: String) -> export::Customer {
        let ctx = get_context();
        export::get_handle_pool(self.port_id).create_handle_customer(DangerousCustomer {
            port_id: self.port_id,
            psychopath: true,
        })
    }
}

struct DangerousCustomer {
    port_id: PortId,
    psychopath: bool,
}

impl Customer for DangerousCustomer {
    fn add_criminal_record(&self, name: String, record: String) {
        get_context().custom.customers.lock().unwrap().get_mut(&name).unwrap().1.push(record);
    }

    fn reform(&self, name: String) -> bool {
        if self.psychopath {
            // Reforming a psychopath is of course impossible, and even will trigger him to kill someone!
            get_context().custom.bank.as_ref().unwrap().ask_nearest_police_station().kill_the_police();
            return false
        }
        if get_context().custom.customers.lock().unwrap().get(&name).unwrap().0 == 0 {
            // He is so poort that he just refuses the be reformed.
            return false
        }
        true
    }

    fn provoke(&self, name: String) -> export::Customer {
        let ctx = get_context();
        export::get_handle_pool(self.port_id).create_handle_customer(DangerousCustomer {
            port_id: self.port_id,
            psychopath: true,
        })
    }
}

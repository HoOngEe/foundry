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

pub const TEMPORARY_PATH: &str = "./tmp";

pub struct DirectoryReserver {
    path: String,
}

impl DirectoryReserver {
    pub fn new(path: String) -> Self {
        std::fs::remove_dir_all(TEMPORARY_PATH).ok(); // we don't care whether it succeeds
        std::fs::create_dir(&path).unwrap();
        DirectoryReserver {
            path,
        }
    }
}

impl Drop for DirectoryReserver {
    fn drop(&mut self) {
        std::fs::remove_dir(&self.path).unwrap();
    }
}

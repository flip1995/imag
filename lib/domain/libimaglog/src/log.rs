//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2019 Matthias Beyer <mail@beyermatthias.de> and contributors
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; version
// 2.1 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//

use libimagdiary::entry::DiaryEntry;
use libimagstore::store::Entry;

use failure::Fallible as Result;
use failure::Error;

use toml::Value;
use toml_query::read::TomlValueReadTypeExt;
use toml_query::insert::TomlValueInsertExt;

pub trait Log : DiaryEntry {
    fn is_log(&self) -> Result<bool>;
    fn make_log_entry(&mut self) -> Result<()>;
}

impl Log for Entry {
    fn is_log(&self) -> Result<bool> {
        self.get_header().read_bool("log.is_log").map(|v| v.unwrap_or(false)).map_err(Error::from)
    }

    fn make_log_entry(&mut self) -> Result<()> {
        self.get_header_mut()
            .insert("log.is_log", Value::Boolean(true))
            .map_err(Error::from)
            .map(|_| ())
    }

}


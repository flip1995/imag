
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

//! Event types
//!
//! This module contains types that represent events. These types (which wrap FileLockEntries from
//! the Store) represent Events, thus they have functionality for settings the start and end-time,
//! getting the start and end time and also deleting start and end time.
//!

use chrono::naive::NaiveDateTime;

use libimagstore::store::Entry;
use libimagerror::errors::ErrorMsg as EM;

use crate::tag::TimeTrackingTag as TTT;
use crate::constants::*;

use toml::Value;
use toml_query::delete::TomlValueDeleteExt;
use toml_query::insert::TomlValueInsertExt;
use toml_query::read::TomlValueReadTypeExt;
use failure::Fallible as Result;
use failure::ResultExt;
use failure::Error;

pub trait TimeTracking {

    fn get_timetrack_tag(&self) -> Result<TTT>;

    fn set_start_datetime(&mut self, dt: NaiveDateTime) -> Result<()>;

    fn get_start_datetime(&self) -> Result<Option<NaiveDateTime>>;

    fn delete_start_datetime(&mut self) -> Result<()>;

    fn set_end_datetime(&mut self, dt: NaiveDateTime) -> Result<()>;

    fn get_end_datetime(&self) -> Result<Option<NaiveDateTime>>;

    fn delete_end_datetime(&mut self) -> Result<()>;

    fn valid(&self) -> Result<bool>;

}

impl TimeTracking for Entry {

    fn get_timetrack_tag(&self) -> Result<TTT> {
        self.get_header()
            .read_string(DATE_TIME_TAG_HEADER_PATH)
            .context(format_err!("Failed to read header '{}' of {}", DATE_TIME_TAG_HEADER_PATH,
                                 self.get_location()))
            .map_err(Error::from)?
            .ok_or_else(|| Error::from(EM::EntryHeaderReadError))
            .map(Into::into)
    }

    fn set_start_datetime(&mut self, dt: NaiveDateTime) -> Result<()> {
        let s = dt.format(DATE_TIME_FORMAT).to_string();

        self.get_header_mut()
            .insert(DATE_TIME_START_HEADER_PATH, Value::String(s))
            .context(format_err!("Failed get header '{}' of {}", DATE_TIME_START_HEADER_PATH,
                                 self.get_location()))
            .map_err(Error::from)
            .map(|_| ())
    }

    fn get_start_datetime(&self) -> Result<Option<NaiveDateTime>> {
        self.get_header()
            .read_string(DATE_TIME_START_HEADER_PATH)
            .context(format_err!("Failed read header '{}' of {}", DATE_TIME_START_HEADER_PATH,
                                 self.get_location()))
            .map_err(Error::from)
            .and_then(header_value_to_dt)
    }

    fn delete_start_datetime(&mut self) -> Result<()> {
        self.get_header_mut()
            .delete(DATE_TIME_START_HEADER_PATH)
            .context(format_err!("Failed delete header '{}' of {}", DATE_TIME_START_HEADER_PATH,
                                 self.get_location()))
            .map_err(Error::from)
            .map(|_| ())
    }

    fn set_end_datetime(&mut self, dt: NaiveDateTime) -> Result<()> {
        let s = dt.format(DATE_TIME_FORMAT).to_string();

        self.get_header_mut()
            .insert(DATE_TIME_END_HEADER_PATH, Value::String(s))
            .context(format_err!("Failed insert header '{}' in {}", DATE_TIME_END_HEADER_PATH,
                                 self.get_location()))
            .map_err(Error::from)
            .map(|_| ())
    }

    fn get_end_datetime(&self) -> Result<Option<NaiveDateTime>> {
        self.get_header()
            .read_string(DATE_TIME_END_HEADER_PATH)
            .context(format_err!("Failed read header '{}' of {}", DATE_TIME_END_HEADER_PATH,
                                 self.get_location()))
            .map_err(Error::from)
            .and_then(header_value_to_dt)
    }

    fn delete_end_datetime(&mut self) -> Result<()> {
        self.get_header_mut()
            .delete(DATE_TIME_END_HEADER_PATH)
            .context(format_err!("Failed delete header '{}' of {}", DATE_TIME_END_HEADER_PATH,
                                 self.get_location()))
            .map_err(Error::from)
            .map(|_| ())
    }

    /// Check whether the Event is valid
    ///
    /// That is:
    ///
    /// * The end date is after the start date (or not set)
    ///
    /// # Return values
    ///
    /// Ok(true) if Event is valid
    /// Ok(false) if Event is invalid
    /// Err(e) if checking validity failed
    ///
    fn valid(&self) -> Result<bool> {
        self.get_start_datetime().and_then(|st| self.get_end_datetime().map(|et| st <= et))
    }

}

fn header_value_to_dt(val: Option<String>) -> Result<Option<NaiveDateTime>> {
    match val {
        Some(ref s) => NaiveDateTime::parse_from_str(s, DATE_TIME_FORMAT)
            .context(format_err!("Failed to parse '{}' datetime with format '{}'",
                                 s, DATE_TIME_FORMAT))
            .map_err(Error::from).map(Some),
        None => Ok(None),
    }
}


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

use std::cmp::Ordering;

use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;
use libimagerror::trace::trace_error;
use libimagentryutil::isa::Is;

use chrono::offset::Local;
use chrono::Datelike;
use itertools::Itertools;
use chrono::naive::NaiveDateTime;
use chrono::Timelike;
use failure::Fallible as Result;
use failure::ResultExt;
use failure::Error;

use crate::entry::IsDiaryEntry;
use crate::diaryid::DiaryId;
use crate::diaryid::FromStoreId;
use crate::iter::DiaryEntryIterator;
use crate::iter::DiaryNameIterator;

pub trait Diary {

    // create or get a new entry for today
    fn new_entry_today(&self, diary_name: &str) -> Result<FileLockEntry>;

    // create or get a new entry for now
    fn new_entry_now(&self, diary_name: &str) -> Result<FileLockEntry>;

    fn new_entry_at(&self, diary_name: &str, ndt: &NaiveDateTime) -> Result<FileLockEntry>;

    // Get an iterator for iterating over all entries of a Diary
    fn entries(&self, diary_name: &str) -> Result<DiaryEntryIterator>;

    fn get_youngest_entry_id(&self, diary_name: &str) -> Option<Result<DiaryId>>;

    /// Get all diary names
    fn diary_names(&self) -> Result<DiaryNameIterator>;

}

impl Diary for Store {

    // create or get a new entry for today
    fn new_entry_today(&self, diary_name: &str) -> Result<FileLockEntry> {
        let dt  = Local::now();
        let ndt = dt.naive_local();
        let id  = DiaryId::new(String::from(diary_name), ndt.year(), ndt.month(), ndt.day(), 0, 0, 0);

        let mut entry = self.retrieve(id)?;
        let _         = entry.set_isflag::<IsDiaryEntry>()?;
        Ok(entry)
    }

    fn new_entry_now(&self, diary_name: &str) -> Result<FileLockEntry> {
        let dt  = Local::now();
        let ndt = dt.naive_local();
        self.new_entry_at(diary_name, &ndt)
    }

    fn new_entry_at(&self, diary_name: &str, ndt: &NaiveDateTime) -> Result<FileLockEntry> {
        let id  = DiaryId::new(String::from(diary_name),
                               ndt.year(),
                               ndt.month(),
                               ndt.day(),
                               ndt.hour(),
                               ndt.minute(),
                               ndt.second());

        let mut entry = self.retrieve(id)?;
        let _         = entry.set_isflag::<IsDiaryEntry>()?;
        Ok(entry)
    }

    // Get an iterator for iterating over all entries
    fn entries(&self, diary_name: &str) -> Result<DiaryEntryIterator> {
        debug!("Building iterator for module 'diary' with diary name = '{}'", diary_name);
        Store::entries(self)
            .map(|iter| DiaryEntryIterator::new(String::from(diary_name), iter.into_storeid_iter()))
    }

    /// get the id of the youngest entry
    fn get_youngest_entry_id(&self, diary_name: &str) -> Option<Result<DiaryId>> {
        match Diary::entries(self, diary_name) {
            Err(e) => Some(Err(e)),
            Ok(entries) => {
                entries.map(|ent| {
                        ent.and_then(|id| DiaryId::from_storeid(&id))
                    })
                    .sorted_by(|a, b| {
                        match (a, b) {
                            (&Ok(ref a), &Ok(ref b)) => {
                                let a : NaiveDateTime = a.clone().into();
                                let b : NaiveDateTime = b.clone().into();

                                a.cmp(&b)
                            },

                            (&Ok(_), &Err(ref e))  => {
                                trace_error(e);
                                Ordering::Less
                            },
                            (&Err(ref e), &Ok(_))  => {
                                trace_error(e);
                                Ordering::Greater
                            },
                            (&Err(ref e1), &Err(ref e2)) => {
                                trace_error(e1);
                                trace_error(e2);
                                Ordering::Equal
                            },
                        }
                    })
                    .into_iter()
                    .rev()
                    .next()
            }
        }
    }

    /// Get all diary names
    fn diary_names(&self) -> Result<DiaryNameIterator> {
        self.entries()
            .map(|it| DiaryNameIterator::new(it.into_storeid_iter()))
            .context("Failed building DiaryNameIteator from entries iterator")
            .map_err(Error::from)
    }

}


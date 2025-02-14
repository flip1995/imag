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

use clap::ArgMatches;
use chrono::NaiveDateTime;
use chrono::Local;
use chrono::Timelike;
use failure::Error;
use failure::ResultExt;
use failure::err_msg;

use libimagdiary::diary::Diary;
use libimagentryedit::edit::Edit;
use libimagrt::runtime::Runtime;
use libimagerror::trace::MapErrTrace;
use libimagerror::exit::ExitUnwrap;
use libimagutil::warn_exit::warn_exit;
use libimagutil::debug_result::DebugResult;
use libimagutil::debug_option::DebugOption;
use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;

use crate::util::get_diary_name;
use crate::util::get_diary_timed_config;
use crate::util::Timed;

pub fn create(rt: &Runtime) {
    let diaryname = get_diary_name(rt)
        .unwrap_or_else( || warn_exit("No diary selected. Use either the configuration file or the commandline option", 1));

    let mut entry = create_entry(rt.store(), &diaryname, rt);

    let _ = rt.report_touched(entry.get_location()).unwrap_or_exit();

    let res = if rt.cli().subcommand_matches("create").unwrap().is_present("no-edit") {
        debug!("Not editing new diary entry");
        Ok(())
    } else {
        debug!("Editing new diary entry");
        entry.edit_content(rt).context(err_msg("Diary edit error")).map_err(Error::from)
    };

    let _ = res.map_err_trace_exit_unwrap();
    info!("Ok!");
}

fn create_entry<'a>(diary: &'a Store, diaryname: &str, rt: &Runtime) -> FileLockEntry<'a> {
    use crate::util::parse_timed_string;

    let create = rt.cli().subcommand_matches("create").unwrap();

    create.value_of("timed")
        .map(|t| parse_timed_string(t, diaryname).map_err_trace_exit_unwrap())
        .map(Some)
        .unwrap_or_else(|| {
            match get_diary_timed_config(rt, diaryname).map_err_trace_exit_unwrap() {
                Some(t) => Some(t),
                None    => {
                    warn!("Missing config: 'diary.diaries.{}.timed'", diaryname);
                    warn!("Assuming 'false'");
                    None
                }
            }
        })
        .map(|timed| {
            let time = create_id_from_clispec(&create, timed);
            diary.new_entry_at(&diaryname, &time)
                .context(err_msg("Store write error"))
                .map_err(Error::from)
        })
        .unwrap_or_else(|| {
            debug!("Creating non-timed entry");
            diary.new_entry_today(diaryname)
        })
        .map_dbg(|e| format!("Created: {}", e.get_location()))
        .map_err_trace_exit_unwrap()
}


fn create_id_from_clispec(create: &ArgMatches, timed_type: Timed) -> NaiveDateTime {
    use std::str::FromStr;

    let dt  = Local::now();
    let ndt = dt.naive_local();

    match timed_type {
        Timed::Daily => {
            debug!("Creating daily-timed entry");
            ndt.with_hour(0)
                .unwrap() // safe because hour = 0 is safe
                .with_minute(0)
                .unwrap() // safe because minute = 0 is safe
                .with_second(0)
                .unwrap() // safe because second = 0 is safe
        },
        Timed::Hourly => {
            debug!("Creating hourly-timed entry");
            ndt.with_minute(0)
                .unwrap() // safe because minute = 0 is safe
                .with_second(0)
                .unwrap() // safe because second = 0 is safe
        },

        Timed::Minutely => {
            let min = create
                .value_of("minute")
                .map_dbg(|m| format!("minute = {:?}", m))
                .and_then(|s| {
                    FromStr::from_str(s)
                        .map_err(|_| warn!("Could not parse minute: '{}'", s))
                        .ok()
                })
                .unwrap_or(ndt.minute());

            ndt.with_minute(min)
                .unwrap_or_else(|| {
                    error!("Cannot set {} as minute, would yield invalid time!", min);
                    ::std::process::exit(1)
                })
                .with_second(0)
                .unwrap() // safe because second = 0 is safe
        },

        Timed::Secondly => {
            let min = create
                .value_of("minute")
                .map_dbg(|m| format!("minute = {:?}", m))
                .and_then(|s| {
                    FromStr::from_str(s)
                        .map_err(|_| warn!("Could not parse minute: '{}'", s))
                        .ok()
                })
                .unwrap_or(ndt.minute());

            let sec = create
                .value_of("second")
                .map_dbg(|s| format!("second = {:?}", s))
                .and_then(|s| {
                    FromStr::from_str(s)
                        .map_err(|_| warn!("Could not parse second: '{}'", s))
                        .ok()
                })
                .unwrap_or(ndt.second());

            ndt.with_minute(min)
                .unwrap_or_else(|| {
                    error!("Cannot set {} as minute, would yield invalid time!", min);
                    ::std::process::exit(1)
                })
                .with_second(sec)
                .unwrap_or_else(|| {
                    error!("Cannot set {} as second, would yield invalid time!", sec);
                    ::std::process::exit(1)
                })
        },
    }
}


//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015, 2016 Matthias Beyer <mail@beyermatthias.de> and contributors
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

use std::process::exit;

use chrono::naive::NaiveDateTime;

use libimagdiary::diaryid::DiaryId;
use libimagrt::runtime::Runtime;
use libimagerror::trace::trace_error_exit;
use libimagtimeui::datetime::DateTime;
use libimagtimeui::parse::Parse;
use libimagutil::warn_exit::warn_exit;
use libimagstore::storeid::IntoStoreId;

use util::get_diary_name;

pub fn delete(rt: &Runtime) {
    use libimaginteraction::ask::ask_bool;

    let diaryname = get_diary_name(rt)
        .unwrap_or_else(|| warn_exit("No diary selected. Use either the configuration file or the commandline option", 1));

    let datetime : Option<NaiveDateTime> = rt
        .cli()
        .subcommand_matches("delete")
        .unwrap()
        .value_of("datetime")
        .map(|dt| { debug!("DateTime = {:?}", dt); dt })
        .and_then(DateTime::parse)
        .map(|dt| dt.into());

    let to_del = match datetime {
        Some(dt) => {
            let id = match DiaryId::from_datetime(diaryname.clone(), dt).into_storeid() {
                Ok(id) => id,
                Err(e) => trace_error_exit(&e, 1),
            };
            Some(rt.store().retrieve(id))
        },
        None     => {
            warn!("Not deleting entries, because missing date/time specification");
            exit(1);
        },
    };

    let to_del = match to_del {
        Some(Ok(e)) => e,

        Some(Err(e)) => trace_error_exit(&e, 1),
        None => warn_exit("No entry", 1)
    };

    let location = to_del.get_location().clone();
    drop(to_del);

    if !ask_bool(&format!("Deleting {:?}", location), Some(true)) {
        info!("Aborting delete action");
        return;
    }

    if let Err(e) = rt.store().delete(location) {
        trace_error_exit(&e, 1)
    }

    info!("Ok!");
}


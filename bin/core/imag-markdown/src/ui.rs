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

use std::path::PathBuf;

use clap::{Arg, ArgMatches, App};
use failure::Fallible as Result;

use libimagstore::storeid::StoreId;
use libimagstore::storeid::IntoStoreId;
use libimagrt::runtime::IdPathProvider;

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .arg(Arg::with_name("links")
             .long("links")
             .short("l")
             .takes_value(false)
             .required(false)
             .multiple(false)
             .help("Print only the links that can be found in the markdown"))

        .arg(Arg::with_name("entry")
             .index(1)
             .takes_value(true)
             .required(false)
             .multiple(true)
             .help("The entries to process")
             .value_name("ENTRY"))
}

pub struct PathProvider;
impl IdPathProvider for PathProvider {
    fn get_ids(matches: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
        matches.values_of("entry")
            .map(|v| v
                 .into_iter()
                 .map(PathBuf::from)
                 .map(|pb| pb.into_storeid())
                 .collect::<Result<Vec<_>>>()
            )
            .transpose()
    }
}


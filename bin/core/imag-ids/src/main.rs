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

#![forbid(unsafe_code)]

#![deny(
    non_camel_case_types,
    non_snake_case,
    path_statements,
    trivial_numeric_casts,
    unstable_features,
    unused_allocation,
    unused_import_braces,
    unused_imports,
    unused_must_use,
    unused_mut,
    unused_qualifications,
    while_true,
)]

extern crate clap;
#[macro_use] extern crate log;
extern crate toml;
extern crate toml_query;
#[macro_use] extern crate failure;

#[cfg(test)]
extern crate env_logger;

extern crate libimagerror;
extern crate libimagstore;
#[macro_use] extern crate libimagrt;

use std::io::Write;

use libimagstore::storeid::StoreId;
use libimagrt::setup::generate_runtime_setup;
use libimagerror::trace::MapErrTrace;
use libimagerror::iter::TraceIterator;
use libimagerror::exit::ExitUnwrap;
use libimagerror::io::ToExitCode;

mod ui;

use crate::ui::build_ui;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-ids",
                                    &version,
                                    "print all ids",
                                    build_ui);

    let print_storepath = rt.cli().is_present("print-storepath");

    let iterator = if rt.ids_from_stdin() {
        debug!("Fetching IDs from stdin...");
        let ids = rt
            .ids::<crate::ui::PathProvider>()
            .map_err_trace_exit_unwrap()
            .unwrap_or_else(|| {
                error!("No ids supplied");
                ::std::process::exit(1);
            });
        Box::new(ids.into_iter().map(Ok))
            as Box<Iterator<Item = Result<StoreId, _>>>
    } else {
        Box::new(rt.store().entries().map_err_trace_exit_unwrap())
            as Box<Iterator<Item = Result<StoreId, _>>>
    }
    .trace_unwrap_exit()
    .map(|id| if print_storepath {
        (Some(rt.store().path()), id)
    } else {
        (None, id)
    });

    let mut stdout = rt.stdout();
    trace!("Got output: {:?}", stdout);

    iterator.for_each(|(storepath, id)| {
        rt.report_touched(&id).unwrap_or_exit();
        if !rt.output_is_pipe() {
            let id = id.to_str().map_err_trace_exit_unwrap();
            trace!("Writing to {:?}", stdout);

            let result = if let Some(store) = storepath {
                writeln!(stdout, "{}/{}", store.display(), id)
            } else {
                writeln!(stdout, "{}", id)
            };

            result.to_exit_code().unwrap_or_exit();
        }
    })
}


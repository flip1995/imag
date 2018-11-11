//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2018 Matthias Beyer <mail@beyermatthias.de> and contributors
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
extern crate failure;

#[macro_use] extern crate libimagrt;
extern crate libimagmail;
extern crate libimagerror;
extern crate libimagutil;

use std::io::Write;

use failure::Error;
use failure::err_msg;

use libimagerror::trace::{MapErrTrace, trace_error};
use libimagerror::iter::TraceIterator;
use libimagerror::exit::ExitUnwrap;
use libimagerror::io::ToExitCode;
use libimagmail::mail::Mail;
use libimagrt::runtime::Runtime;
use libimagrt::setup::generate_runtime_setup;
use libimagutil::info_result::*;

mod ui;

use ui::build_ui;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-mail",
                                    &version,
                                    "Mail collection tool",
                                    build_ui);

    rt.cli()
        .subcommand_name()
        .map(|name| {
            debug!("Call {}", name);
            match name {
                "import-mail" => import_mail(&rt),
                "list"        => list(&rt),
                "mail-store"  => mail_store(&rt),
                other         => {
                    debug!("Unknown command");
                    let _ = rt.handle_unknown_subcommand("imag-mail", other, rt.cli())
                        .map_err_trace_exit_unwrap(1)
                        .code()
                        .map(::std::process::exit);
                }
            }
        });
}

fn import_mail(rt: &Runtime) {
    let scmd = rt.cli().subcommand_matches("import-mail").unwrap();
    let path = scmd.value_of("path").unwrap(); // enforced by clap

    let mail = Mail::import_from_path(rt.store(), path)
        .map_info_str("Ok")
        .map_err_trace_exit_unwrap(1);

    let _ = rt.report_touched(mail.fle().get_location()).map_err_trace_exit_unwrap(1);
}

fn list(rt: &Runtime) {
    use failure::ResultExt;

        // TODO: Implement lister type in libimagmail for this
    fn list_mail(rt: &Runtime, m: Mail) {
        let id = match m.get_message_id() {
            Ok(Some(f)) => f,
            Ok(None) => "<no id>".to_owned(),
            Err(e) => {
                trace_error(&e);
                "<error>".to_owned()
            },
        };

        let from = match m.get_from() {
            Ok(Some(f)) => f,
            Ok(None) => "<no from>".to_owned(),
            Err(e) => {
                trace_error(&e);
                "<error>".to_owned()
            },
        };

        let to = match m.get_to() {
            Ok(Some(f)) => f,
            Ok(None) => "<no to>".to_owned(),
            Err(e) => {
                trace_error(&e);
                "<error>".to_owned()
            },
        };

        let subject = match m.get_subject() {
            Ok(Some(f)) => f,
            Ok(None) => "<no subject>".to_owned(),
            Err(e) => {
                trace_error(&e);
                "<error>".to_owned()
            },
        };

        writeln!(rt.stdout(),
                 "Mail: {id}\n\tFrom: {from}\n\tTo: {to}\n\t{subj}\n",
                 from = from,
                 id   = id,
                 subj = subject,
                 to   = to
        ).to_exit_code().unwrap_or_exit();

        let _ = rt.report_touched(m.fle().get_location()).map_err_trace_exit_unwrap(1);
    }

    let _ = rt.store()
        .entries()
        .map_err_trace_exit_unwrap(1)
        .trace_unwrap_exit(1)
        .filter(|id| id.is_in_collection(&["mail"]))
        .filter_map(|id| {
            rt.store()
                .get(id)
                .context(err_msg("Ref handling error"))
                .map_err(Error::from)
                .map_err_trace_exit_unwrap(1)
                .map(|fle| Mail::from_fle(fle).map_err_trace().ok())
        })
        .filter_map(|e| e)
        .for_each(|m| list_mail(&rt, m));
}

fn mail_store(rt: &Runtime) {
    let _ = rt.cli().subcommand_matches("mail-store").unwrap();
    error!("This feature is currently not implemented.");
    unimplemented!()
}


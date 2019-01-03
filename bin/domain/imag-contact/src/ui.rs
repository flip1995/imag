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

use clap::{Arg, App, SubCommand};

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .subcommand(SubCommand::with_name("list")
                   .about("List contacts")
                   .version("0.1")
                   .arg(Arg::with_name("filter")
                        .index(1)
                        .takes_value(true)
                        .required(false)
                        .multiple(true)
                        .value_name("FILTER")
                        .help("Filter by these properties (not implemented yet)"))
                   .arg(Arg::with_name("format")
                        .long("format")
                        .takes_value(true)
                        .required(false)
                        .multiple(false)
                        .value_name("FORMAT")
                        .help("Format to format the listing"))
                   .arg(Arg::with_name("json")
                        .long("json")
                        .takes_value(false)
                        .required(false)
                        .multiple(false)
                        .help("Print output as JSON"))
                   )

        .subcommand(SubCommand::with_name("import")
                   .about("Import contacts")
                   .version("0.1")
                   .arg(Arg::with_name("path")
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .multiple(false)
                        .value_name("PATH")
                        .help("Import from this file/directory"))
                   )

        .subcommand(SubCommand::with_name("show")
                   .about("Show contact")
                   .version("0.1")
                   .arg(Arg::with_name("hash")
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .multiple(false)
                        .value_name("HASH")
                        .help("Show the contact pointed to by this reference hash"))
                   .arg(Arg::with_name("format")
                        .long("format")
                        .takes_value(true)
                        .required(false)
                        .multiple(false)
                        .value_name("FORMAT")
                        .help("Format to format the contact when printing it"))
                   )

        .subcommand(SubCommand::with_name("find")
                   .about("Find contact by grepping for a string (no regex yet)")
                   .version("0.1")
                   .arg(Arg::with_name("string")
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .multiple(true)
                        .value_name("grepstring")
                        .help("Find the contact by grepping for this string"))

                   .arg(Arg::with_name("find-show")
                        .long("show")
                        .short("s")
                        .takes_value(false)
                        .required(false)
                        .multiple(false)
                        .help("Show the result")
                        .conflicts_with("find-list")
                        )

                   .arg(Arg::with_name("find-list")
                        .long("list")
                        .short("l")
                        .takes_value(false)
                        .required(false)
                        .multiple(false)
                        .help("List the result")
                        .conflicts_with("find-show")
                        )

                   .arg(Arg::with_name("json")
                        .long("json")
                        .takes_value(false)
                        .required(false)
                        .multiple(false)
                        .help("Print output as JSON")
                        .conflicts_with("format")
                        .conflicts_with("find-show")
                        .conflicts_with("find-list"))

                   .arg(Arg::with_name("format")
                        .long("format")
                        .takes_value(true)
                        .required(false)
                        .multiple(false)
                        .value_name("FORMAT")
                        .help("Format to format the contact when printing it")
                        .conflicts_with("json")
                        .conflicts_with("find-show")
                        .conflicts_with("find-list"))

                   .arg(Arg::with_name("find-id")
                        .long("id")
                        .takes_value(false)
                        .required(false)
                        .multiple(false)
                        .help("Print the store ids of the found entries")
                        .conflicts_with("find-full-id")
                        .conflicts_with("json")
                        .conflicts_with("find-show")
                        .conflicts_with("find-list"))

                   .arg(Arg::with_name("find-full-id")
                        .long("full-id")
                        .takes_value(false)
                        .required(false)
                        .multiple(false)
                        .help("Print the path to the file in the store of the found entries")
                        .conflicts_with("find-id")
                        .conflicts_with("json")
                        .conflicts_with("find-show")
                        .conflicts_with("find-list"))


                   )

        .subcommand(SubCommand::with_name("create")
                   .about("Create a contact file (.vcf) and track it in imag.")
                   .version("0.1")
                   .arg(Arg::with_name("file-location")
                        .short("F")
                        .long("file")
                        .takes_value(true)
                        .required(false)
                        .multiple(false)
                        .value_name("PATH")
                        .help("Create this file. If a directory is passed, a file with a uuid as name will be created. vcf contents are dumped to stdout if this is not passed."))
                   .arg(Arg::with_name("dont-track")
                        .short("T")
                        .long("no-track")
                        .takes_value(false)
                        .required(false)
                        .multiple(false)
                        .help("Don't track the new vcf file if one is created."))
                   )
}

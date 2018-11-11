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

#![recursion_limit="256"]

#![deny(
    dead_code,
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

#[macro_use] extern crate log;
extern crate itertools;
extern crate toml;
extern crate toml_query;

#[macro_use] extern crate libimagstore;
extern crate libimagerror;
#[macro_use] extern crate libimagentryutil;
extern crate failure;

module_entry_path_mod!("ref");

pub mod reference;
pub mod refstore;

#[cfg(feature  = "generators-sha1")]
extern crate sha1;

#[cfg(any(
    feature = "generators-sha224",
    feature = "generators-sha256",
    feature = "generators-sha384",
    feature = "generators-sha512",
))]
extern crate sha2;

#[cfg(feature  = "generators-sha3")]
extern crate sha3;

#[cfg(any(
    feature = "generators-sha1",
    feature = "generators-sha224",
    feature = "generators-sha256",
    feature = "generators-sha384",
    feature = "generators-sha512",
    feature = "generators-sha3",
))]
extern crate hex;

#[cfg(feature = "generators")]
pub mod generators;


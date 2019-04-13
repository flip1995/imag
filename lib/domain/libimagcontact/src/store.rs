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

use toml::Value;
use toml::to_string as toml_to_string;
use toml::from_str as toml_from_str;
use toml_query::insert::TomlValueInsertExt;
use vobject::vcard::Vcard;
use failure::Error;
use failure::Fallible as Result;

use libimagstore::storeid::StoreId;
use libimagstore::iter::Entries;
use libimagstore::store::Store;
use libimagstore::store::FileLockEntry;
use libimagentryutil::isa::Is;

use contact::IsContact;
use deser::DeserVcard;
use util;

pub trait ContactStore<'a> {

    // creating

    fn create_from_path(&'a self, p: &PathBuf)   -> Result<FileLockEntry<'a>>;
    fn retrieve_from_path(&'a self, p: &PathBuf) -> Result<FileLockEntry<'a>>;

    fn create_from_buf(&'a self, buf: &str)      -> Result<FileLockEntry<'a>>;
    fn retrieve_from_buf(&'a self, buf: &str)    -> Result<FileLockEntry<'a>>;

    // getting

    fn all_contacts(&'a self) -> Result<Entries<'a>>;
}

/// The extension for the Store to work with contacts
impl<'a> ContactStore<'a> for Store {

    fn create_from_path(&'a self, p: &PathBuf) -> Result<FileLockEntry<'a>> {
        util::read_to_string(p).and_then(|buf| self.create_from_buf(&buf))
    }

    fn retrieve_from_path(&'a self, p: &PathBuf) -> Result<FileLockEntry<'a>> {
        util::read_to_string(p).and_then(|buf| self.retrieve_from_buf(&buf))
    }

    /// Create contact ref from buffer
    fn create_from_buf(&'a self, buf: &str) -> Result<FileLockEntry<'a>> {
        let (sid, value) = prepare_fetching_from_store(buf)?;
        postprocess_fetched_entry(self.create(sid)?, value)
    }

    fn retrieve_from_buf(&'a self, buf: &str) -> Result<FileLockEntry<'a>> {
        let (sid, value) = prepare_fetching_from_store(buf)?;
        postprocess_fetched_entry(self.retrieve(sid)?, value)
    }

    fn all_contacts(&'a self) -> Result<Entries<'a>> {
        self.entries().map(|ent| ent.in_collection("contact"))
    }

}

/// Prepare the fetching from the store.
///
/// That means calculating the StoreId and the Value from the vcard data
fn prepare_fetching_from_store(buf: &str) -> Result<(StoreId, Value)> {
    let vcard = Vcard::build(&buf).map_err(Error::from)?;
    debug!("Parsed: {:?}", vcard);

    let uid = vcard.uid()
        .ok_or_else(|| Error::from(format_err!("UID Missing: {}", buf.to_string())))?;

    let value = { // dirty ugly hack
        let serialized = DeserVcard::from(vcard);
        let serialized = toml_to_string(&serialized)?;
        toml_from_str::<Value>(&serialized)?
    };

    let sid = ::module_path::new_id(uid.raw())?;

    Ok((sid, value))
}

/// Postprocess the entry just fetched from the store
fn postprocess_fetched_entry<'a>(mut entry: FileLockEntry<'a>, value: Value) -> Result<FileLockEntry<'a>> {
    entry.set_isflag::<IsContact>()?;
    entry.get_header_mut().insert("contact.data", value)?;
    Ok(entry)
}


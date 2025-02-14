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
use std::sync::Arc;
use std::fmt::Debug;

use failure::Fallible as Result;

use crate::storeid::StoreIdWithBase;
use crate::file_abstraction::FileAbstraction;

/// See documentation for PathIterator
pub(crate) trait PathIterBuilder : Debug {
    fn build_iter(&self) -> Box<Iterator<Item = Result<PathBuf>>>;
    fn in_collection(&mut self, c: &str) -> Result<()>;
}

/// A wrapper for an iterator over `PathBuf`s
///
/// As the backend defines how "iterating over all entries" is implemented, this type holds a
/// `PathIterBuilder` internally. This type is used to create new iterator instances every time the
/// "settings" for how the iterator behaves are changed. This basically means: If the PathIterator
/// is requested to not iterate over a directory "a" but rather its subdirectory "a/b", the
/// implementation asks the `PathIterBuilder` to create a new iterator for that.
///
/// The `PathIterBuilder` can then yield a new iterator instance which is optimized for the new
/// requirements (which basically means: Construct a new WalkDir object which does traverse the
/// subdirectory instead of the parent).
///
/// This means quite a few allocations down the road, as the PathIterator itself is not generic, but
/// this seems to be the best way to implement this.
pub(crate) struct PathIterator<'a> {
    iter_builder: Box<PathIterBuilder>,
    iter:         Box<Iterator<Item = Result<PathBuf>>>,
    storepath:    &'a PathBuf,
    backend:      Arc<FileAbstraction>,
}

impl<'a> PathIterator<'a> {

    pub fn new(iter_builder: Box<PathIterBuilder>,
               storepath: &'a PathBuf,
               backend: Arc<FileAbstraction>)
        -> PathIterator<'a>
    {
        trace!("Generating iterator object with PathIterBuilder: {:?}", iter_builder);
        let iter = iter_builder.build_iter();
        PathIterator { iter_builder, iter, storepath, backend }
    }

    pub fn in_collection(mut self, c: &str) -> Result<Self> {
        trace!("Generating iterator object for collection: {}", c);
        self.iter_builder.in_collection(c)?;
        self.iter = self.iter_builder.build_iter();
        trace!("Set new iterator");
        Ok(self)
    }

    /// Turn iterator into its internals
    ///
    /// Used for `Entries::into_storeid_iter()`
    ///
    /// # TODO
    ///
    /// Revisit whether this can be done in a cleaner way. See commit message for why this is
    /// needed.
    pub(crate) fn into_inner(self) -> Box<Iterator<Item = Result<PathBuf>>> {
        self.iter
    }

}

impl<'a> Iterator for PathIterator<'a> {
    type Item = Result<StoreIdWithBase<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next() {
            trace!("Iterating over item = {:?}", next);
            match next {
                Err(e)   => return Some(Err(e)),
                Ok(next) => match self.backend.is_file(&next) {
                    Err(e)    => return Some(Err(e)),
                    Ok(true)  => return Some(StoreIdWithBase::from_full_path(&self.storepath, next)),
                    Ok(false) => { continue },
                }
            }
        }

        None
    }

}


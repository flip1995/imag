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

use std::io::Write;

use libimagstore::store::Entry;

use crate::viewer::Viewer;
use crate::error::Result;

pub struct PlainViewer {
    show_header: bool
}

impl PlainViewer {

    pub fn new(show_header: bool) -> PlainViewer {
        PlainViewer {
            show_header: show_header,
        }
    }

}

impl Viewer for PlainViewer {

    fn view_entry<W>(&self, e: &Entry, sink: &mut W) -> Result<()>
        where W: Write
    {
        if self.show_header {
            let _ = writeln!(sink, "{}", e.get_header())?;
        }
        let _ = writeln!(sink, "{}", e.get_content())?;
        Ok(())
    }

}

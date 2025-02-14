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

use std::collections::BTreeMap;

use failure::Fallible as Result;
use failure::ResultExt;
use failure::Error;
use crate::link::extract_links;

use libimagentryurl::linker::UrlLinker;
use libimagentrylink::linkable::Linkable;
use libimagentryref::reference::MutRef;
use libimagentryref::reference::RefFassade;
use libimagentryref::hasher::sha1::Sha1Hasher;
use libimagstore::store::Entry;
use libimagstore::store::Store;
use libimagstore::storeid::StoreId;
use libimagerror::errors::ErrorMsg;

use std::path::PathBuf;

use url::Url;

/// A link Processor which collects the links from a Markdown and passes them on to
/// `libimagentrylink` functionality
///
/// The processor can be configured to
///
///  * Process internal links (from store entry to store entry)
///  * Process internal links with automatically creating targets
///     If an internal link is encountered, the corrosponding target must be present in the store.
///     If it is not, it will either be created or the processing fails
///  * Process external links (from store entry to URL)
///  * Process refs (from store entry to files on the filesystem and outside of the store)
///  (default: false)
///
///  # Note
///
///  There's no LinkProcessor::new() function, please use `LinkProcessor::default()`.
///
pub struct LinkProcessor {
    process_links: bool,
    create_targets: bool,
    process_urls: bool,
    process_refs: bool
}

impl LinkProcessor {

    /// Switch internal link processing on/off
    ///
    /// Internal links are links which are simply `dirctory/file`, but not `/directory/file`, as
    /// beginning an id with `/` is discouraged in imag.
    pub fn process_links(mut self, b: bool) -> Self {
        self.process_links = b;
        self
    }

    /// Switch internal link target creation on/off
    ///
    /// If a link points to a non-existing imag entry, a `false` here will cause the processor to
    /// return an error from `process()`. A `true` setting will create the entry and then fetch it
    /// to link it to the processed entry.
    pub fn create_targets(mut self, b: bool) -> Self {
        self.create_targets = b;
        self
    }

    /// Switch external link processing on/off
    ///
    /// An external link must start with `https://` or `http://`.
    pub fn process_urls(mut self, b: bool) -> Self {
        self.process_urls = b;
        self
    }

    /// Switch ref processing on/off
    ///
    /// A Ref is to be expected beeing a link with `file::///` at the beginning.
    pub fn process_refs(mut self, b: bool) -> Self {
        self.process_refs = b;
        self
    }

    /// Process an Entry for its links
    ///
    ///
    /// # Notice
    ///
    /// Whenever a "ref" is created, that means when a URL points to a filesystem path (normally
    /// when using `file:///home/user/foobar.file` for example), the _current_ implementation uses
    /// libimagentryref to create make the entry into a ref.
    ///
    /// The configuration of the `libimagentryref::reference::Reference::make_ref()` call is as
    /// follows:
    ///
    /// * Name of the collection: "root"
    /// * Configuration: `{"root": "/"}`
    ///
    /// This implementation might change in the future, so that the configuration and the name of
    /// the collection can be passed to the function, or in a way that the user is asked what to do
    /// during the runtime of this function.
    ///
    ///
    /// # Warning
    ///
    /// When `LinkProcessor::create_targets()` was called to set the setting to true, this
    /// function returns all errors returned by the Store.
    ///
    /// That means:
    ///
    /// * For an internal link, the linked target is created if create_targets() is true,
    ///   else error
    /// * For an external link, if create_targets() is true, libimagentrylink creates the
    ///   external link entry, else the link is ignored
    /// * all other cases do not create elements in the store
    ///
    pub fn process<'a>(&self, entry: &mut Entry, store: &'a Store) -> Result<()> {
        let text = entry.to_str()?;
        trace!("Processing: {:?}", entry.get_location());
        for link in extract_links(&text).into_iter() {
            trace!("Processing {:?}", link);
            match LinkQualification::qualify(&link.link) {
                LinkQualification::InternalLink => {
                    if !self.process_links {
                        continue
                    }

                    let id         = StoreId::new(PathBuf::from(&link.link))?;
                    let mut target = if self.create_targets {
                        store.retrieve(id)?
                    } else {
                        store.get(id.clone())?
                            .ok_or_else(|| Error::from(format_err!("Store get error: {}", id)))?
                    };

                    let _ = entry.add_link(&mut target)?;
                },
                LinkQualification::ExternalLink(url) => {
                    if !self.process_urls {
                        continue
                    }

                    entry.add_url(store, url)?;
                },
                LinkQualification::RefLink(url) => {
                    use sha1::{Sha1, Digest};

                    if !self.process_refs {
                        trace!("Not processing refs... continue...");
                        continue
                    }

                    // because we can make one entry only into _one_ ref, but a markdown document
                    // might contain several "ref" links, we create a new entry for the ref we're
                    // about to create
                    //
                    // We generate the StoreId with the SHA1 hash of the path, which is the best
                    // option we have
                    // right now
                    //
                    // TODO: Does this make sense? Can we improve this?
                    let path         = url.host_str().unwrap_or_else(|| url.path());
                    let path         = PathBuf::from(path);
                    let ref_entry_id = {
                        let digest = Sha1::digest(path.to_str().ok_or(ErrorMsg::UTF8Error)?.as_bytes());
                        StoreId::new(PathBuf::from(format!("ref/{:x}", digest)))? // TODO: Ugh...
                    };
                    let mut ref_entry = store.retrieve(ref_entry_id)?;

                    let ref_collection_name = "root";

                    // TODO: Maybe this can be a const?
                    // TODO: Maybe we need this ot be overrideable? Not sure.
                    let ref_collection_config = {
                        let mut map = BTreeMap::new();
                        map.insert(String::from("root"), PathBuf::from("/"));
                        ::libimagentryref::reference::Config::new(map)
                    };

                    trace!("URL            = {:?}", url);
                    trace!("URL.path()     = {:?}", url.path());
                    trace!("URL.host_str() = {:?}", url.host_str());

                    trace!("Processing ref: {:?} -> {path}, collection: {ref_collection_name}, cfg: {cfg:?}",
                           path                = path.display(),
                           ref_collection_name = ref_collection_name,
                           cfg                 = ref_collection_config);

                    ref_entry.as_ref_with_hasher_mut::<Sha1Hasher>()
                        .make_ref(path,
                                  ref_collection_name,
                                  &ref_collection_config,
                                  false)?;

                    trace!("Ready processing, linking new ref entry...");

                    let _ = entry.add_link(&mut ref_entry)?;
                },
                LinkQualification::Undecidable(e) => {
                    // error
                    return Err(e)
                        .context(format_err!("Undecidable link type: {}", link.link.clone()))
                        .map_err(Error::from)
                },
            }
        }

        Ok(())
    }

}

/// Enum to tell what kind of link a string of text is
enum LinkQualification {
    InternalLink,
    ExternalLink(Url),
    RefLink(Url),
    Undecidable(Error),
}

impl LinkQualification {
    fn qualify(text: &str) -> LinkQualification {
        trace!("Qualifying: {}", text);
        match Url::parse(text) {
            Ok(url) => {
                if url.scheme() == "file" {
                    trace!("Qualifying = RefLink");
                    return LinkQualification::RefLink(url)
                }

                // else we assume the following, as other stuff gets thrown out by
                // url::Url::parse() as Err(_)
                //
                // if url.scheme() == "https" || url.scheme() == "http" {
                    return LinkQualification::ExternalLink(url);
                // }
            },

            Err(e) => {
                match e {
                    ::url::ParseError::RelativeUrlWithoutBase => {
                        trace!("Qualifying = InternalLink");
                        LinkQualification::InternalLink
                    },

                    _ => LinkQualification::Undecidable(Error::from(e)),
                }
            }
        }
    }
}

impl Default for LinkProcessor {
    fn default() -> Self {
        LinkProcessor {
            process_links: true,
            create_targets: false,
            process_urls: true,
            process_refs: false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use libimagstore::store::Store;
    use libimagentrylink::linkable::Linkable;

    fn setup_logging() {
        let _ = ::env_logger::try_init();
    }

    pub fn get_store() -> Store {
        Store::new_inmemory(PathBuf::from("/"), &None).unwrap()
    }

    #[test]
    fn test_process_no_links() {
        setup_logging();
        let store = get_store();

        let mut base = store.create(PathBuf::from("test-1")).unwrap();
        *base.get_content_mut() = format!("This is an example entry with no links");

        let update = store.update(&mut base);
        assert!(update.is_ok());

        let processor = LinkProcessor::default();

        let result = processor.process(&mut base, &store);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_one_existing_file_linked() {
        setup_logging();
        let store = get_store();

        let mut base = store.create(PathBuf::from("test-2.1")).unwrap();
        *base.get_content_mut() = format!("This is an example entry with one [link](test-2.2)");

        let update = store.update(&mut base);
        assert!(update.is_ok());

        // immediately drop as we don't need this entry right now
        let _ = store.create(PathBuf::from("test-2.2")).unwrap();

        let processor = LinkProcessor::default()
            .process_links(true)
            .create_targets(false)
            .process_urls(false)
            .process_refs(false);

        let result = processor.process(&mut base, &store);
        assert!(result.is_ok(), "Should be Ok(()): {:?}", result);

        {
            let base_links = base.links();
            assert!(base_links.is_ok());
            let base_links : Vec<_> = base_links.unwrap().collect();

            assert_eq!(1, base_links.len());
            assert_eq!("test-2.2", base_links[0].to_str().unwrap());
        }

        {
            let link = store.get(PathBuf::from("test-2.2")).unwrap().unwrap();
            let link_links = link.links();
            assert!(link_links.is_ok());
            let link_links : Vec<_> = link_links.unwrap().collect();

            assert_eq!(1, link_links.len());
            assert_eq!("test-2.1", link_links[0].to_str().unwrap());
        }
    }

    #[test]
    fn test_process_one_existing_file_linked_faulty() {
        setup_logging();
        let store = get_store();

        let mut base = store.create(PathBuf::from("test-2.1")).unwrap();
        *base.get_content_mut() = format!("This is an example entry with one [link](/test-2.2)");

        let update = store.update(&mut base);
        assert!(update.is_ok());

        let processor = LinkProcessor::default()
            .process_links(true)
            .create_targets(false)
            .process_urls(false)
            .process_refs(false);

        let result = processor.process(&mut base, &store);
        assert!(result.is_err(), "Should be Err(_), but is Ok(())");
    }

    #[test]
    fn test_process_one_nonexisting_file_linked() {
        setup_logging();
        let store = get_store();

        let mut base = store.create(PathBuf::from("test-2.1")).unwrap();
        *base.get_content_mut() = format!("This is an example entry with one [link](test-2.2)");

        let update = store.update(&mut base);
        assert!(update.is_ok());

        let processor = LinkProcessor::default()
            .process_links(true)
            .create_targets(true)
            .process_urls(false)
            .process_refs(false);

        let result = processor.process(&mut base, &store);
        assert!(result.is_ok(), "Should be Ok(()): {:?}", result);

        {
            let base_links = base.links();
            assert!(base_links.is_ok());
            let base_links : Vec<_> = base_links.unwrap().collect();

            assert_eq!(1, base_links.len());
            assert_eq!("test-2.2", base_links[0].to_str().unwrap());
        }

        {
            let link = store.get(PathBuf::from("test-2.2")).unwrap().unwrap();
            let link_links = link.links();
            assert!(link_links.is_ok());
            let link_links : Vec<_> = link_links.unwrap().collect();

            assert_eq!(1, link_links.len());
            assert_eq!("test-2.1", link_links[0].to_str().unwrap());
        }
    }

    #[test]
    fn test_process_one_url() {
        setup_logging();
        let store = get_store();

        let mut base = store.create(PathBuf::from("test-5.1")).unwrap();
        *base.get_content_mut() = format!("An [example](http://example.com) is here.");

        let update = store.update(&mut base);
        assert!(update.is_ok());

        let processor = LinkProcessor::default()
            .process_links(true)
            .create_targets(true)
            .process_urls(true)
            .process_refs(false);

        let result = processor.process(&mut base, &store);
        assert!(result.is_ok(), "Should be Ok(()): {:?}", result);

        // The hash of "http://example.com" processed in the `libimagentrylink` way.
        let expected_link = "url/9c17e047f58f9220a7008d4f18152fee4d111d14";
        {
            let base_links = base.links();
            assert!(base_links.is_ok());
            let base_links : Vec<_> = base_links.unwrap().collect();

            assert_eq!(1, base_links.len());
            assert_eq!(expected_link, base_links[0].to_str().unwrap());
        }

        let entries = store.entries();
        assert!(entries.is_ok());
        let entries : Vec<_> = entries.unwrap().into_storeid_iter().collect();

        assert_eq!(2, entries.len(), "Expected 2 links, got: {:?}", entries);

        {
            let link = store.get(PathBuf::from(expected_link)).unwrap().unwrap();
            let link_links = link.links();
            assert!(link_links.is_ok());
            let link_links : Vec<_> = link_links.unwrap().collect();

            assert_eq!(1, link_links.len());
            assert_eq!("test-5.1", link_links[0].to_str().unwrap());
        }
    }

    #[test]
    fn test_process_one_ref() {
        setup_logging();
        let store = get_store();

        let mut base = store.create(PathBuf::from("test-5.1")).unwrap();

        // As the ref target must exist, we're using /etc/hosts here
        *base.get_content_mut() = format!("An [example ref](file:///etc/hosts) is here.");

        let update = store.update(&mut base);
        assert!(update.is_ok());

        let processor = LinkProcessor::default()
            .process_links(false)
            .create_targets(false)
            .process_urls(false)
            .process_refs(true);

        let result = processor.process(&mut base, &store);
        assert!(result.is_ok(), "Should be Ok(()): {:?}", result);

        let entries = store.entries();
        assert!(entries.is_ok());
        let entries : Vec<_> = entries.unwrap().into_storeid_iter().collect();

        assert_eq!(2, entries.len(), "Expected 1 entries, got: {:?}", entries);
        debug!("{:?}", entries);
    }

    #[test]
    fn test_process_two_refs() {
        setup_logging();
        let store = get_store();

        let mut base = store.create(PathBuf::from("test-5.1")).unwrap();

        // As the ref target must exist, we're using /etc/hosts here
        *base.get_content_mut() = format!(
            r#"An [example ref](file:///etc/hosts)
            is [here](file:///etc/group)."#
        );

        let update = store.update(&mut base);
        assert!(update.is_ok());

        let processor = LinkProcessor::default()
            .process_links(false)
            .create_targets(false)
            .process_urls(false)
            .process_refs(true);

        let result = processor.process(&mut base, &store);
        assert!(result.is_ok(), "Should be Ok(()): {:?}", result);

        let entries = store.entries();
        assert!(entries.is_ok());
        let entries : Vec<_> = entries.unwrap().into_storeid_iter().collect();

        assert_eq!(3, entries.len(), "Expected 3 links, got: {:?}", entries);
        debug!("{:?}", entries);
    }

    #[test]
    fn test_process_refs_with_ref_processing_switched_off() {
        setup_logging();
        let store = get_store();

        let mut base = store.create(PathBuf::from("test-5.1")).unwrap();

        // As the ref target must exist, we're using /etc/hosts here
        *base.get_content_mut() = format!(
            r#"An [example ref](file:///etc/hosts)
            is [here](file:///etc/group)."#
        );

        let update = store.update(&mut base);
        assert!(update.is_ok());

        let processor = LinkProcessor::default()
            .process_links(false)
            .create_targets(false)
            .process_urls(false)
            .process_refs(false);

        let result = processor.process(&mut base, &store);
        assert!(result.is_ok(), "Should be Ok(()): {:?}", result);

        let entries = store.entries();
        assert!(entries.is_ok());
        let entries : Vec<_> = entries.unwrap().into_storeid_iter().collect();

        assert_eq!(1, entries.len(), "Expected 1 entries, got: {:?}", entries);
        debug!("{:?}", entries);
    }

    #[test]
    fn test_process_external_link_with_external_link_processing_switched_off() {
        setup_logging();
        let store = get_store();

        let mut base = store.create(PathBuf::from("test-5.1")).unwrap();
        *base.get_content_mut() = format!("An [example](http://example.com) is here.");

        let update = store.update(&mut base);
        assert!(update.is_ok());

        let processor = LinkProcessor::default()
            .process_links(true)
            .create_targets(true)
            .process_urls(false)
            .process_refs(false);

        let result = processor.process(&mut base, &store);
        assert!(result.is_ok(), "Should be Ok(()): {:?}", result);

        let entries = store.entries();
        assert!(entries.is_ok());
        let entries : Vec<_> = entries.unwrap().into_storeid_iter().collect();

        assert_eq!(1, entries.len(), "Expected 1 entries, got: {:?}", entries);
    }

    #[test]
    fn test_process_one_existing_file_linked_with_processing_switched_off() {
        setup_logging();
        let store = get_store();

        let mut base = store.create(PathBuf::from("test-2.1")).unwrap();
        *base.get_content_mut() = format!("This is an example entry with one [link](test-2.2)");

        let update = store.update(&mut base);
        assert!(update.is_ok());

        // immediately drop as we don't need this entry right now
        let _ = store.create(PathBuf::from("test-2.2")).unwrap();

        let processor = LinkProcessor::default()
            .process_links(false)
            .create_targets(false)
            .process_urls(false)
            .process_refs(false);

        let result = processor.process(&mut base, &store);
        assert!(result.is_ok(), "Should be Ok(()): {:?}", result);

        assert_eq!(2, store.entries().unwrap().collect::<Vec<_>>().len());
    }

}


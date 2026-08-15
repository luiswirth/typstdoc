use std::collections::HashMap;
use std::sync::Mutex;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::FontBook;
use typst::utils::LazyHash;
use typst::{Feature, Features, Library, LibraryExt, World};

use crate::files::Files;
use crate::fonts::Fonts;

/// The world every fragment of a crate is compiled in.
///
/// Only the main file changes from one fragment to the next, so the library,
/// the fonts, the files read so far and comemo's memoization all carry over.
pub struct FragmentWorld {
    library: LazyHash<Library>,
    fonts: Fonts,
    files: Files,
    main: Source,
    cache: Mutex<Cache>,
}

#[derive(Default)]
struct Cache {
    bytes: HashMap<FileId, FileResult<Bytes>>,
    sources: HashMap<FileId, FileResult<Source>>,
}

impl FragmentWorld {
    pub fn new(files: Files, fonts: Fonts) -> Self {
        let library = Library::builder()
            .with_features(Features::from_iter([Feature::Html]))
            .build();

        // The id is unique rather than interned by path, so that a crate's own
        // `fragment.typ`, if it has one, stays reachable. Its virtual path still
        // decides where a relative path inside a fragment resolves from.
        let vpath = VirtualPath::new("fragment.typ").expect("a valid path");
        let main = FileId::unique(RootedPath::new(VirtualRoot::Project, vpath));

        Self {
            library: LazyHash::new(library),
            fonts,
            files,
            main: Source::new(main, String::new()),
            cache: Mutex::new(Cache::default()),
        }
    }

    /// Replaces the source the next compilation takes as its main file.
    pub fn set_main(&mut self, text: String) {
        self.main = Source::new(self.main.id(), text);
    }
}

impl World for FragmentWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            return Ok(self.main.clone());
        }
        let mut cache = self.cache.lock().unwrap();
        if let Some(source) = cache.sources.get(&id) {
            return source.clone();
        }
        let source = self.files.read(id).and_then(|bytes| {
            let text = std::str::from_utf8(&bytes).map_err(|_| FileError::InvalidUtf8)?;
            Ok(Source::new(id, text.trim_start_matches('\u{feff}').into()))
        });
        cache.sources.insert(id, source.clone());
        source
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main.id() {
            return Ok(Bytes::from_string(self.main.text().to_string()));
        }
        let mut cache = self.cache.lock().unwrap();
        if let Some(bytes) = cache.bytes.get(&id) {
            return bytes.clone();
        }
        let bytes = self.files.read(id);
        cache.bytes.insert(id, bytes.clone());
        bytes
    }

    fn font(&self, index: usize) -> Option<typst::text::Font> {
        self.fonts.get(index)
    }

    /// No date, so that the same sources always document the same.
    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}

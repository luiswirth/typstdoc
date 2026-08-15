use std::path::{Path, PathBuf};
use std::sync::Arc;

use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;
use typst::syntax::{FileId, VirtualRoot};

/// The file system a fragment is compiled against.
///
/// What a build can reach is a property of the build and not of typstdoc:
/// a docs.rs build is sandboxed and offline, where a local `cargo doc`
/// reaches the packages the author has installed.
/// Resolution is therefore a function the caller supplies.
#[derive(Clone)]
pub struct Files(Arc<dyn Fn(FileId) -> FileResult<Bytes> + Send + Sync>);

impl Files {
    pub fn new(read: impl Fn(FileId) -> FileResult<Bytes> + Send + Sync + 'static) -> Self {
        Self(Arc::new(read))
    }

    /// Resolves nothing, the identity of [`Files::or`].
    pub fn none() -> Self {
        Self::new(|id| Err(not_found(id)))
    }

    /// Resolves project files against a directory, and no package.
    pub fn directory(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self::new(move |id| match id.root() {
            VirtualRoot::Project => read(&id.vpath().realize(&root)?),
            VirtualRoot::Package(_) => Err(not_found(id)),
        })
    }

    /// Resolves packages against a directory laid out as
    /// `<root>/<namespace>/<name>/<version>`, which is how Typst
    /// stores them under its data directory, and no project file.
    pub fn packages(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self::new(move |id| match id.root() {
            VirtualRoot::Package(spec) => {
                let root = root
                    .join(spec.namespace.as_str())
                    .join(spec.name.as_str())
                    .join(spec.version.to_string());
                read(&id.vpath().realize(&root)?)
            }
            VirtualRoot::Project => Err(not_found(id)),
        })
    }

    /// Falls back to `other` wherever `self` finds no file.
    ///
    /// An error other than a miss stops the search, so a permission problem
    /// surfaces rather than turning into a missing file further along.
    pub fn or(self, other: Files) -> Self {
        Self::new(move |id| match self.read(id) {
            Err(FileError::NotFound(_)) => other.read(id),
            result => result,
        })
    }

    pub fn read(&self, id: FileId) -> FileResult<Bytes> {
        (self.0)(id)
    }
}

fn read(path: &Path) -> FileResult<Bytes> {
    std::fs::read(path)
        .map(Bytes::new)
        .map_err(|err| FileError::from_io(err, path))
}

fn not_found(id: FileId) -> FileError {
    FileError::NotFound(id.vpath().get_without_slash().into())
}

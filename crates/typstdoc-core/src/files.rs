use std::path::{Path, PathBuf};

use typst::diag::{FileError, FileResult};
use typst::foundations::Bytes;
use typst::syntax::{FileId, VirtualRoot};

/// The file system a fragment is compiled against.
///
/// What a build can reach is a property of the build and not of typstdoc:
/// a docs.rs build is sandboxed and offline, where a local `cargo doc` reaches
/// the packages the author has installed.
pub trait Files: Send + Sync {
    /// Reads the file the id names.
    ///
    /// A file that is not there is a [`FileError::NotFound`], which is what
    /// tells a miss from a failure when several places are searched.
    fn read(&self, id: FileId) -> FileResult<Bytes>;
}

/// Reads files from directories, in Typst's own layout.
///
/// A project file lives under a project root at its own path, and a package
/// under a package root at `<root>/<namespace>/<name>/<version>`. Typst keeps
/// installed packages under its data directory and downloaded ones under its
/// cache directory, so a build usually has more than one package root.
///
/// The default reads nothing, having nowhere to read from.
#[derive(Debug, Default, Clone)]
pub struct Directories {
    pub project: Option<PathBuf>,
    /// Searched in order.
    pub packages: Vec<PathBuf>,
}

impl Files for Directories {
    fn read(&self, id: FileId) -> FileResult<Bytes> {
        self.roots(id.root())
            .into_iter()
            .map(|root| read(&id.vpath().realize(&root)?))
            .find(|result| !matches!(result, Err(FileError::NotFound(_))))
            .unwrap_or_else(|| Err(not_found(id)))
    }
}

impl Directories {
    /// The package roots Typst reads on this machine, and no project root.
    ///
    /// An installed package takes precedence over a downloaded one, and each
    /// root lies where Typst's environment says or under its data and cache
    /// directory. A root that is not there resolves nothing, which is what a
    /// sandboxed build sees.
    pub fn installed() -> Self {
        let packages = [
            ("TYPST_PACKAGE_PATH", dirs::data_dir()),
            ("TYPST_PACKAGE_CACHE_PATH", dirs::cache_dir()),
        ]
        .into_iter()
        .filter_map(|(variable, directory)| match std::env::var_os(variable) {
            Some(path) => Some(PathBuf::from(path)),
            None => Some(directory?.join("typst").join("packages")),
        })
        .collect();

        Self {
            project: None,
            packages,
        }
    }

    /// The directories a file with the given root may lie in, in search order.
    fn roots(&self, root: &VirtualRoot) -> Vec<PathBuf> {
        match root {
            VirtualRoot::Project => self.project.iter().cloned().collect(),
            VirtualRoot::Package(spec) => self
                .packages
                .iter()
                .map(|root| {
                    root.join(spec.namespace.as_str())
                        .join(spec.name.as_str())
                        .join(spec.version.to_string())
                })
                .collect(),
        }
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

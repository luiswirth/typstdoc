use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use typstdoc_core::{Directories, Fonts, Renderer};

use crate::Result;
use crate::shim;
use crate::source;

/// Copies the package a crate root belongs to, with the Typst in the doc
/// comments of its sources rendered, and answers where the crate root lies in
/// the copy.
///
/// The whole package is copied rather than its sources alone, so that a file a
/// source includes from beside it is there as well.
pub fn render(root: &Path) -> Result<PathBuf> {
    let root = root.canonicalize()?;
    let package =
        package(&root).ok_or_else(|| format!("no package holds {}", root.display()))?;
    let staged = staging(package)?;

    let mut renderer = renderer(package)?;
    copy(package, &staged, &mut renderer)?;

    Ok(staged.join(root.strip_prefix(package)?))
}

/// The directory of the package a file belongs to.
fn package(file: &Path) -> Option<&Path> {
    file.ancestors()
        .skip(1)
        .find(|directory| directory.join("Cargo.toml").is_file())
}

/// Where a package is copied to, emptied of what a previous run left.
///
/// The place a package is staged in follows from where it lies, so that the
/// pages of one documentation build point at the same sources as the next.
fn staging(package: &Path) -> Result<PathBuf> {
    let mut hasher = DefaultHasher::new();
    package.hash(&mut hasher);
    let name = package.file_name().unwrap_or("crate".as_ref());

    let staged = std::env::temp_dir()
        .join("typstdoc")
        .join(format!("{}-{:016x}", name.display(), hasher.finish()));

    if staged.exists() {
        std::fs::remove_dir_all(&staged)?;
    }
    Ok(staged)
}

fn copy(from: &Path, to: &Path, renderer: &mut Renderer) -> Result<()> {
    std::fs::create_dir_all(to)?;

    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name();
        let (from, to) = (entry.path(), to.join(&name));

        if entry.file_type()?.is_dir() {
            // What a package builds into is not what it is built from.
            if name != "target" && name != ".git" {
                copy(&from, &to, renderer)?;
            }
        } else if from.extension() == Some("rs".as_ref()) {
            std::fs::write(&to, source::render(&from, renderer)?)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }

    Ok(())
}

/// The renderer the package's fragments are compiled through.
///
/// The package is the project, so a fragment reaches the Typst files that lie
/// beside the sources it is written in.
fn renderer(package: &Path) -> Result<Renderer> {
    let files = Directories {
        project: Some(package.to_owned()),
        ..Directories::installed()
    };

    let preamble = match preamble(package) {
        Some(path) => std::fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?,
        None => String::new(),
    };

    Ok(Renderer::new(files, Fonts::embedded(), preamble))
}

/// The preamble the package's fragments are compiled with.
///
/// A workspace is one body of notation, so the preamble is looked for above
/// the package as well, and the nearest one holds.
fn preamble(package: &Path) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os(shim::PREAMBLE) {
        return Some(path.into());
    }

    package
        .ancestors()
        .map(|directory| directory.join(source::PREAMBLE))
        .find(|path| path.is_file())
}

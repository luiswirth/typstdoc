use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::Result;

/// The stylesheet as it is written beside the staged package.
const FILE: &str = "typstdoc.css";

/// How rustdoc is told about a stylesheet.
const OPTION: &str = "--extend-css";
const SHORT: &str = "-e";
const JOINED: &str = "--extend-css=";

/// Hands rustdoc the stylesheet the rendered equations are set in.
///
/// rustdoc copies it to the root of the documentation and links it from every
/// page at that page's depth, which is what makes one stylesheet reach pages
/// that lie at different depths.
///
/// It takes a single stylesheet, so one that was asked for already is kept and
/// ours is written after it.
pub fn extend(args: &mut Vec<OsString>, directory: &Path) -> Result<()> {
    let given = given(args);

    let mut css = String::new();
    if let Some(given) = &given {
        css.push_str(&std::fs::read_to_string(&given.path)?);
        if !css.ends_with('\n') {
            css.push('\n');
        }
    }
    css.push_str(&typstdoc_core::stylesheet());

    let path = directory.join(FILE);
    std::fs::write(&path, css)?;

    match given {
        Some(given) if given.joined => args[given.index] = joined(&path),
        Some(given) => args[given.index] = path.into(),
        None => {
            args.push(OPTION.into());
            args.push(path.into());
        }
    }

    Ok(())
}

/// A stylesheet rustdoc was asked for already.
struct Given {
    /// The argument carrying its path, which is the one to write over.
    index: usize,
    /// Whether that argument carries the option as well.
    joined: bool,
    path: PathBuf,
}

fn given(args: &[OsString]) -> Option<Given> {
    for (index, arg) in args.iter().enumerate() {
        match arg.to_str() {
            Some(OPTION | SHORT) => {
                return args.get(index + 1).map(|path| Given {
                    index: index + 1,
                    joined: false,
                    path: path.into(),
                });
            }
            Some(arg) if arg.starts_with(JOINED) => {
                return Some(Given {
                    index,
                    joined: true,
                    path: arg[JOINED.len()..].into(),
                });
            }
            _ => {}
        }
    }

    None
}

fn joined(path: &Path) -> OsString {
    let mut arg = OsString::from(JOINED);
    arg.push(path);
    arg
}

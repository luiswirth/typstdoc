use std::fmt::{self, Display, Formatter};

use typst::diag::SourceDiagnostic;
use typst::ecow::EcoVec;

/// Why a fragment did not render.
#[derive(Debug)]
pub enum Error {
    /// The Typst compiler rejected the fragment.
    Compile(EcoVec<SourceDiagnostic>),
    /// The compiled document is not shaped the way typstdoc reads a fragment
    /// out of it, which means typst-html changed under us.
    UnexpectedOutput,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Compile(diagnostics) => {
                let mut diagnostics = diagnostics.iter();
                match diagnostics.next() {
                    Some(first) => write!(f, "{}", first.message)?,
                    None => write!(f, "compilation failed")?,
                }
                for diagnostic in diagnostics {
                    write!(f, "; {}", diagnostic.message)?;
                }
                Ok(())
            }
            Self::UnexpectedOutput => {
                write!(f, "typst produced a document typstdoc cannot read")
            }
        }
    }
}

impl std::error::Error for Error {}

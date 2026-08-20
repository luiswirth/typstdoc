use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use typst::foundations::Bytes;
use typst::text::Font;

use crate::fonts::embedded_files;

/// The family the stylesheet names, and the font file carrying it.
const FAMILY: &str = "New Computer Modern Math";
const FILE: &str = "NewCMMath-Regular";

/// The stylesheet that decides how a page's equations look.
///
/// MathML carries no font information, so a page that links nothing is set in
/// whatever the browser resolves the `math` generic to. The font travels in
/// the stylesheet rather than beside it, since the only thing a documentation
/// build can hand rustdoc is a file of CSS.
///
/// Without the embedded fonts there is nothing to carry, and the rule alone is
/// still what a reader who has the family installed needs.
pub fn stylesheet() -> String {
    let mut css = String::new();
    if let Some(font) = math_font() {
        css.push_str(&face(&font));
    }
    css.push_str(&format!("math {{ font-family: \"{FAMILY}\", math; }}\n"));
    css
}

/// The rule that gives the browser the font itself.
fn face(font: &Font) -> String {
    let encoded = STANDARD.encode(font.data());
    format!(
        "@font-face {{ font-family: \"{FAMILY}\"; \
         src: url(\"data:font/otf;base64,{encoded}\") format(\"opentype\"); }}\n"
    )
}

/// The embedded font the stylesheet carries.
///
/// New Computer Modern ships a Book and a Regular of the same family and
/// weight, which no metadata tells apart, so the file is named outright. The
/// Regular is the one drawn for the screen.
fn math_font() -> Option<Font> {
    embedded_files()
        .flat_map(|data| Font::iter(Bytes::new(data)))
        .find(|font| font.post_script_name().as_deref() == Some(FILE))
}

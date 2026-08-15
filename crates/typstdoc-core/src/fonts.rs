use typst::foundations::Bytes;
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;

/// The fonts a fragment is compiled against.
///
/// MathML carries no font information, so these decide nothing about how an
/// equation looks. Typst still needs them to resolve the text it is given.
pub struct Fonts {
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

impl Fonts {
    pub fn new(fonts: Vec<Font>) -> Self {
        let book = FontBook::from_fonts(&fonts);
        Self {
            book: LazyHash::new(book),
            fonts,
        }
    }

    /// The fonts compiled into the binary.
    ///
    /// Empty unless the `embedded-fonts` feature is on.
    pub fn embedded() -> Self {
        let fonts = typst_assets::fonts()
            .flat_map(|data| Font::iter(Bytes::new(data)))
            .collect();
        Self::new(fonts)
    }

    pub fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    pub fn get(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }
}

//! The attribute that renders Typst fragments in doc comments, over
//! [`typstdoc_core`].

mod doc;
mod renderer;

use proc_macro::TokenStream;

/// Renders the Typst fragments in the doc comments of an item.
///
/// The macro is handed the whole item, so an attribute on a module reaches
/// every item the module holds, however deeply nested.
///
/// ```
/// #[cfg_attr(doc, typstdoc::typstdoc)]
/// mod hodge {
///     /// The Hodge star $star: Lambda^k -> Lambda^(n-k)$.
///     pub struct Star;
/// }
/// ```
///
/// Under `cfg_attr(doc, ...)` the attribute is stripped in an ordinary build
/// and runs only when rustdoc does.
#[proc_macro_attribute]
pub fn typstdoc(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    doc::rewrite(item)
}

use typst::comemo::Track;
use typst::ecow::EcoString;
use typst::model::LateLinkResolver;
use typst::syntax::SyntaxMode;
use typst_html::{HtmlDocument, HtmlElement, HtmlNode, HtmlOptions, HtmlTag, tag};

use crate::error::Error;
use crate::files::Files;
use crate::fonts::Fonts;
use crate::world::FragmentWorld;

/// Renders the fragments of one crate.
pub struct Renderer {
    world: FragmentWorld,
    preamble: String,
}

impl Renderer {
    /// A renderer over the given files, whose fragments are each compiled with
    /// the preamble before them.
    ///
    /// The preamble is Typst markup, so a `#show` or `#set` rule in it holds
    /// for every fragment, which an import from inside a fragment could not
    /// give. A crate that has nothing to say to all of its fragments passes an
    /// empty one.
    pub fn new(files: impl Files + 'static, fonts: Fonts, preamble: String) -> Self {
        Self {
            world: FragmentWorld::new(files, fonts),
            preamble,
        }
    }

    pub fn render(&mut self, source: &str, mode: SyntaxMode) -> Result<Rendered, Error> {
        self.world.set_main(wrap(&self.preamble, source, mode));
        let compiled = typst::compile::<HtmlDocument>(&self.world);
        let document = compiled.output.map_err(Error::Compile)?;
        extract(&document)
    }
}

/// A rendered fragment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rendered {
    /// The HTML, ready to be spliced into a rustdoc page.
    pub html: String,
    /// What a page carrying this fragment needs beyond that HTML.
    pub assets: Assets,
}

/// The resources a page needs for the fragments on it.
///
/// A fragment reports what it needs rather than emitting it, so that a page
/// carrying many fragments links each resource once.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Assets {
    /// Stylesheets, as Typst writes them into the document head.
    pub styles: Vec<EcoString>,
}

/// Wraps a fragment into the markup that is compiled.
///
/// A Typst file is markup at its top level, so a mode is entered by the
/// delimiters that enter it, and the preamble, being markup itself, stands
/// before them. The fragment keeps whatever spacing it was written with, which
/// is what tells `$x$` from `$ x $`.
fn wrap(preamble: &str, source: &str, mode: SyntaxMode) -> String {
    let fragment = match mode {
        SyntaxMode::Markup => source.into(),
        SyntaxMode::Math => format!("${source}$"),
        SyntaxMode::Code => format!("#{{{source}}}"),
    };
    format!("{preamble}\n{fragment}")
}

/// Reads a fragment out of a compiled document.
///
/// Typst exports a whole page: the fragment is the body, and the resources it
/// needs are what Typst put in the head for it.
fn extract(document: &HtmlDocument) -> Result<Rendered, Error> {
    let root = document.root();
    let head = child(root, tag::head).ok_or(Error::UnexpectedOutput)?;
    let body = child(root, tag::body).ok_or(Error::UnexpectedOutput)?;
    let body = unparagraph(body);

    let styles = head
        .children
        .iter()
        .filter_map(element_of)
        .filter(|element| element.tag == tag::style)
        .filter_map(|element| text(element.children.first()))
        .collect();

    Ok(Rendered {
        html: encode(document, body)?,
        assets: Assets { styles },
    })
}

/// Encodes the children of an element.
///
/// Every encoder typst-html exposes writes a whole document, so the fragment is
/// what remains once the document around it is taken off again. The affixes are
/// constants of the encoder, and failing to find them means it no longer writes
/// what typstdoc reads.
fn encode(document: &HtmlDocument, element: &HtmlElement) -> Result<String, Error> {
    let resolver = LateLinkResolver::new(None, document.introspector().as_ref());
    let options = HtmlOptions { pretty: false };
    let encoded =
        typst_html::html_in_bundle(element, &options, resolver.track()).map_err(Error::Compile)?;

    let tag = element.tag.resolve();
    encoded
        .strip_prefix("<!DOCTYPE html>")
        .and_then(|rest| rest.strip_prefix(&format!("<{tag}>")))
        .and_then(|rest| rest.strip_suffix(&format!("</{tag}>")))
        .map(String::from)
        .ok_or(Error::UnexpectedOutput)
}

/// The element whose children are the fragment.
///
/// Typst writes a whole document, so it puts inline content in the paragraph a
/// page needs. A fragment is spliced into a page that has one already, so that
/// paragraph belongs to the document around it rather than to the fragment.
///
/// A tag is where the document introspects itself and writes nothing, so it is
/// not what makes a fragment more than its paragraph.
fn unparagraph(body: &HtmlElement) -> &HtmlElement {
    let mut content = body
        .children
        .iter()
        .filter(|node| !matches!(node, HtmlNode::Tag(_)));

    match (content.next(), content.next()) {
        (Some(HtmlNode::Element(paragraph)), None) if paragraph.tag == tag::p => paragraph,
        _ => body,
    }
}

fn child(element: &HtmlElement, tag: HtmlTag) -> Option<&HtmlElement> {
    element
        .children
        .iter()
        .filter_map(element_of)
        .find(|child| child.tag == tag)
}

fn element_of(node: &HtmlNode) -> Option<&HtmlElement> {
    match node {
        HtmlNode::Element(element) => Some(element),
        _ => None,
    }
}

fn text(node: Option<&HtmlNode>) -> Option<EcoString> {
    match node? {
        HtmlNode::Text(text, _) => Some(text.clone()),
        _ => None,
    }
}

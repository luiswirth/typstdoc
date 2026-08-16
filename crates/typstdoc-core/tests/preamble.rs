use std::path::PathBuf;

use typstdoc_core::{Directories, Fonts, Renderer, SyntaxMode};

fn render(preamble: &str, source: &str, mode: SyntaxMode) -> Result<String, String> {
    let files = Directories {
        packages: vec![
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join("packages"),
        ],
        ..Default::default()
    };
    Renderer::new(files, Fonts::embedded(), preamble.into())
        .render(source, mode)
        .map(|rendered| rendered.html)
        .map_err(|error| error.to_string())
}

#[test]
fn a_definition_is_in_scope_for_every_fragment() {
    let preamble = "#let sq(x) = $#x^2$";
    assert_eq!(
        render(preamble, "sq(y)", SyntaxMode::Math).unwrap(),
        "<math><msup><mi>𝑦</mi><mn>2</mn></msup></math>"
    );
}

#[test]
fn a_show_rule_holds_for_a_fragment() {
    let preamble = r#"#show "note": strong"#;
    assert_eq!(
        render(preamble, "a note", SyntaxMode::Markup).unwrap(),
        "a <strong>note</strong>"
    );
}

#[test]
fn the_preamble_reads_the_files_a_fragment_reads() {
    let preamble = r#"#import "@local/fixture:0.1.0": squared"#;
    assert_eq!(
        render(preamble, "squared($x$)", SyntaxMode::Code).unwrap(),
        "<math><msup><mi>𝑥</mi><mn>2</mn></msup></math>"
    );
}

#[test]
fn the_preamble_does_not_decide_inline_from_block() {
    let preamble = "#let unused = 1";
    let inline = render(preamble, "x", SyntaxMode::Math).unwrap();
    let block = render(preamble, " x ", SyntaxMode::Math).unwrap();
    assert!(!inline.contains("display"), "{inline}");
    assert!(block.contains(r#"<math display="block">"#), "{block}");
}

#[test]
fn a_preamble_that_does_not_compile_is_an_error() {
    assert!(render("#import \"@local/missing:0.1.0\": *", "x", SyntaxMode::Math).is_err());
}

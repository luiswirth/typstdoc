use std::path::PathBuf;

use typstdoc_core::{Files, Fonts, Renderer, SyntaxMode};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(name)
}

fn render(files: Files, source: &str) -> Result<String, String> {
    Renderer::new(files, Fonts::embedded())
        .render(source, SyntaxMode::Markup)
        .map(|fragment| fragment.html)
        .map_err(|error| error.to_string())
}

const IMPORT_PACKAGE: &str = r#"#import "@local/fixture:0.1.0": squared
#squared($x$)"#;

const IMPORT_PROJECT: &str = r#"#import "/notation.typ": cubed
#cubed($x$)"#;

#[test]
fn a_package_resolves_against_its_root() {
    let html = render(Files::packages(fixture("packages")), IMPORT_PACKAGE).unwrap();
    assert!(html.contains("<msup><mi>𝑥</mi><mn>2</mn></msup>"), "{html}");
}

#[test]
fn a_project_file_resolves_against_its_root() {
    let html = render(Files::directory(fixture("project")), IMPORT_PROJECT).unwrap();
    assert!(html.contains("<msup><mi>𝑥</mi><mn>3</mn></msup>"), "{html}");
}

#[test]
fn the_two_roots_do_not_answer_for_each_other() {
    assert!(render(Files::packages(fixture("project")), IMPORT_PROJECT).is_err());
    assert!(render(Files::directory(fixture("packages")), IMPORT_PACKAGE).is_err());
}

#[test]
fn or_searches_both() {
    let files = Files::directory(fixture("project")).or(Files::packages(fixture("packages")));
    assert!(render(files.clone(), IMPORT_PACKAGE).is_ok());
    assert!(render(files, IMPORT_PROJECT).is_ok());
}

#[test]
fn none_resolves_nothing() {
    assert!(render(Files::none(), IMPORT_PACKAGE).is_err());
    assert!(render(Files::none(), IMPORT_PROJECT).is_err());
    assert!(render(Files::none(), "plain markup").is_ok());
}

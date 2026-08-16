use std::path::PathBuf;

use typstdoc_core::{Directories, Files, Fonts, Renderer, SyntaxMode};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(name)
}

fn render(files: impl Files + 'static, source: &str) -> Result<String, String> {
    Renderer::new(files, Fonts::embedded(), String::new())
        .render(source, SyntaxMode::Markup)
        .map(|rendered| rendered.html)
        .map_err(|error| error.to_string())
}

const IMPORT_PACKAGE: &str = r#"#import "@local/fixture:0.1.0": squared
#squared($x$)"#;

const IMPORT_PROJECT: &str = r#"#import "/notation.typ": cubed
#cubed($x$)"#;

fn project() -> Directories {
    Directories {
        project: Some(fixture("project")),
        ..Default::default()
    }
}

fn packages() -> Directories {
    Directories {
        packages: vec![fixture("packages")],
        ..Default::default()
    }
}

#[test]
fn a_package_resolves_against_its_root() {
    let html = render(packages(), IMPORT_PACKAGE).unwrap();
    assert!(html.contains("<msup><mi>𝑥</mi><mn>2</mn></msup>"), "{html}");
}

#[test]
fn a_project_file_resolves_against_its_root() {
    let html = render(project(), IMPORT_PROJECT).unwrap();
    assert!(html.contains("<msup><mi>𝑥</mi><mn>3</mn></msup>"), "{html}");
}

#[test]
fn the_two_roots_do_not_answer_for_each_other() {
    assert!(render(project(), IMPORT_PACKAGE).is_err());
    assert!(render(packages(), IMPORT_PROJECT).is_err());
}

#[test]
fn both_roots_are_read_from_one_directories() {
    let both = Directories {
        project: Some(fixture("project")),
        packages: vec![fixture("packages")],
    };
    assert!(render(both.clone(), IMPORT_PACKAGE).is_ok());
    assert!(render(both, IMPORT_PROJECT).is_ok());
}

#[test]
fn package_roots_are_searched_in_order() {
    let roots = Directories {
        packages: vec![fixture("project"), fixture("packages")],
        ..Default::default()
    };
    assert!(render(roots, IMPORT_PACKAGE).is_ok());
}

#[test]
fn nowhere_to_read_from_resolves_nothing() {
    assert!(render(Directories::default(), IMPORT_PACKAGE).is_err());
    assert!(render(Directories::default(), IMPORT_PROJECT).is_err());
    assert!(render(Directories::default(), "plain markup").is_ok());
}

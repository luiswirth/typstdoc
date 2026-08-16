use cargo_typstdoc::source::rewrite;
use typstdoc_core::{Directories, Fonts, Renderer};

fn render(source: &str) -> String {
    let mut renderer = Renderer::new(Directories::default(), Fonts::embedded(), String::new());
    rewrite(source, &mut renderer, &mut |_, _, _| {})
}

fn failures(source: &str) -> Vec<(usize, String)> {
    let mut renderer = Renderer::new(Directories::default(), Fonts::embedded(), String::new());
    let mut failures = Vec::new();
    rewrite(source, &mut renderer, &mut |line, fragment, _| {
        failures.push((line, fragment.to_owned()))
    });
    failures
}

#[test]
fn a_fragment_becomes_an_attribute() {
    assert_eq!(
        render("/// On $x$.\npub struct Star;\n"),
        "#[doc = \"On <math><mi>𝑥</mi></math>.\"]\npub struct Star;\n"
    );
}

#[test]
fn what_holds_no_fragment_is_left_alone() {
    let source = "/// A price of $5, and `$x$` and no math.\npub struct Plain;\n";
    assert_eq!(render(source), source);
}

#[test]
fn an_inner_comment_stays_inner() {
    assert!(render("//! On $x$.\n").starts_with("#![doc = "));
}

#[test]
fn a_run_keeps_the_lines_it_stood_on() {
    let source = "/// One $x$,\n/// two,\n/// three.\nstruct S;\n";
    let rendered = render(source);
    assert_eq!(rendered.lines().count(), source.lines().count());
    assert!(rendered.ends_with("struct S;\n"));
}

#[test]
fn a_comment_of_the_other_kind_is_another_run() {
    let rendered = render("//! On $x$.\n\n/// Also $y$.\nstruct S;\n");
    assert_eq!(rendered.matches("#[doc = ").count(), 1);
    assert_eq!(rendered.matches("#![doc = ").count(), 1);
}

#[test]
fn an_item_between_two_comments_is_left_between_them() {
    let rendered = render("/// On $x$.\nstruct A;\n/// On $y$.\nstruct B;\n");
    assert_eq!(rendered.lines().nth(1), Some("struct A;"));
    assert_eq!(rendered.lines().nth(3), Some("struct B;"));
}

#[test]
fn a_nested_item_is_reached() {
    let rendered = render("mod m {\n    /// On $x$.\n    struct S;\n}\n");
    assert!(rendered.contains("    #[doc = "), "{rendered}");
}

#[test]
fn a_fragment_that_does_not_compile_stays_as_it_was() {
    let source = "/// One $x$,\n/// two $#panic()$.\nstruct S;\n";
    assert_eq!(failures(&source.to_string()), vec![(2, "$#panic()$".into())]);
    assert!(render(source).contains("two $#panic()$."), "{}", render(source));
}

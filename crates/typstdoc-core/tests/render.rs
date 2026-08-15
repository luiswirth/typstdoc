use typstdoc_core::{Directories, Fonts, Renderer, SyntaxMode};

fn renderer() -> Renderer {
    Renderer::new(Directories::default(), Fonts::embedded())
}

fn render(source: &str, mode: SyntaxMode) -> String {
    renderer().render(source, mode).unwrap().html
}

#[test]
fn math_is_mathml() {
    assert_eq!(
        render("x^2", SyntaxMode::Math),
        "<p><math><msup><mi>𝑥</mi><mn>2</mn></msup></math></p>"
    );
}

#[test]
fn spacing_decides_inline_from_block() {
    assert!(render(" x ", SyntaxMode::Math).contains(r#"<math display="block">"#));
    assert!(!render("x", SyntaxMode::Math).contains("display"));
}

#[test]
fn markup_is_markup() {
    assert_eq!(
        render("*bold* and $x$", SyntaxMode::Markup),
        "<p><strong>bold</strong> and <math><mi>𝑥</mi></math></p>"
    );
}

#[test]
fn code_is_evaluated() {
    assert_eq!(render(r#"[#(1 + 1)]"#, SyntaxMode::Code), "<p>2</p>");
}

#[test]
fn empty_renders_to_what_the_mode_reads_as_empty() {
    assert_eq!(render("", SyntaxMode::Markup), "");
    assert_eq!(render("", SyntaxMode::Code), "");
    assert_eq!(render("", SyntaxMode::Math), "<p><math></math></p>");
}

#[test]
fn math_reports_the_stylesheet_and_markup_does_not() {
    assert!(
        !renderer()
            .render("x", SyntaxMode::Math)
            .unwrap()
            .assets
            .styles
            .is_empty()
    );
    assert!(
        renderer()
            .render("x", SyntaxMode::Markup)
            .unwrap()
            .assets
            .styles
            .is_empty()
    );
}

#[test]
fn a_renderer_serves_many_fragments() {
    let mut renderer = renderer();
    let first = renderer.render("alpha", SyntaxMode::Math).unwrap().html;
    let second = renderer.render("beta", SyntaxMode::Math).unwrap().html;
    assert_ne!(first, second);
    assert_eq!(
        renderer.render("alpha", SyntaxMode::Math).unwrap().html,
        first
    );
}

#[test]
fn an_error_is_reported_rather_than_panicking() {
    let error = renderer()
        .render("#panic()", SyntaxMode::Markup)
        .unwrap_err();
    assert!(error.to_string().contains("panicked"));
}

#[test]
fn mathml_does_not_depend_on_the_fonts() {
    let with = renderer()
        .render("integral_Omega alpha", SyntaxMode::Math)
        .unwrap();
    let without = Renderer::new(Directories::default(), Fonts::new(Vec::new()))
        .render("integral_Omega alpha", SyntaxMode::Math)
        .unwrap();
    assert_eq!(with, without);
}

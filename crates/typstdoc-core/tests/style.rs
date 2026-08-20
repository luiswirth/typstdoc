use typstdoc_core::stylesheet;

#[test]
fn the_stylesheet_sets_the_math_font() {
    assert!(stylesheet().contains(r#"math { font-family: "New Computer Modern Math", math; }"#));
}

#[test]
fn the_stylesheet_carries_the_font_it_names() {
    let css = stylesheet();
    assert!(css.contains(r#"@font-face { font-family: "New Computer Modern Math";"#));
    assert!(css.contains("data:font/otf;base64,"));
    assert!(css.len() > 1_000_000);
}

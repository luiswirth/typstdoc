use typstdoc_core::{SyntaxMode, scan};

fn sources(doc: &str) -> Vec<&str> {
    scan(doc).into_iter().map(|found| found.source).collect()
}

#[test]
fn a_fragment_is_what_the_delimiters_enclose() {
    assert_eq!(sources("the star $star(alpha)$ of a form"), ["star(alpha)"]);
}

#[test]
fn spacing_is_left_to_typst() {
    assert_eq!(sources("$x$ and $ x $"), ["x", " x "]);
}

#[test]
fn a_code_span_is_text() {
    assert_eq!(sources("run `$ cargo build` first"), [] as [&str; 0]);
    assert_eq!(sources("`$x$`"), [] as [&str; 0]);
}

#[test]
fn a_code_block_is_text() {
    let doc = "before\n\n```rust\nlet price = \"$5\";\n```\n\nafter";
    assert_eq!(sources(doc), [] as [&str; 0]);
}

#[test]
fn an_indented_code_block_is_text() {
    assert_eq!(sources("before\n\n    $x$\n\nafter"), [] as [&str; 0]);
}

#[test]
fn a_fragment_reaches_neither_into_nor_across_a_code_span() {
    assert_eq!(sources("$a `b` c$"), [] as [&str; 0]);
}

#[test]
fn an_escaped_dollar_neither_opens_nor_closes() {
    assert_eq!(sources(r"between \$5 and \$6"), [] as [&str; 0]);
    assert_eq!(sources(r"$a \$ b$"), [r"a \$ b"]);
}

#[test]
fn a_lone_dollar_is_text() {
    assert_eq!(sources("Hello $world."), [] as [&str; 0]);
}

#[test]
fn a_fragment_spans_lines_but_not_a_blank_one() {
    assert_eq!(sources("$ alpha\nwedge beta $"), [" alpha\nwedge beta "]);
    assert_eq!(sources("$ alpha\n\nwedge beta $"), [] as [&str; 0]);
}

#[test]
fn the_range_covers_the_delimiters() {
    let doc = "the star $star(alpha)$ of a form";
    let found = scan(doc).remove(0);
    assert_eq!(&doc[found.range.clone()], "$star(alpha)$");
    assert_eq!(
        &doc[found.range.start + 1..found.range.end - 1],
        found.source
    );
}

#[test]
fn a_fragment_delimited_by_dollars_is_math() {
    assert_eq!(scan("$x$").remove(0).mode, SyntaxMode::Math);
}

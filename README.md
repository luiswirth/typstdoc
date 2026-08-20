# typstdoc

Typst notation in Rust doc comments, compiled to MathML when rustdoc runs.

```rust
/// The Hodge star $star: Lambda^k -> Lambda^(n-k)$ satisfies
/// $ alpha wedge star beta = inner(alpha, beta) vol. $
pub fn hodge_star(form: &Form) -> Form {}
```

Nothing renders in the browser.
The equations are already MathML in the generated pages,
so they stay selectable, reflow with the text, and are read out by screen readers.

## Status

Early.
The path from a doc comment to MathML in a rustdoc page works, both ways in.
The math font and the per-page stylesheet do not exist yet; see the issues.

## Use

Two front ends over one library.

`cargo typstdoc` documents a crate that says nothing about typstdoc:

```
cargo typstdoc --no-deps
```

It runs `cargo doc` with itself in rustdoc's place.
Each time cargo calls it, it copies the package,
renders the fragments in the doc comments of the copy,
and hands rustdoc the copy.
The crate keeps no trace of typstdoc, and its root is reached like every other module.

The attribute renders the doc comments of the item it is written on:

```rust
#[cfg_attr(doc, typstdoc::typstdoc)]
mod hodge {}
```

Under `cfg_attr(doc, ...)` it is stripped in an ordinary build and runs only when rustdoc does.
It is the way in for docs.rs, which runs `cargo doc` itself.

## Design

The unit is a Typst fragment, not an equation.
A fragment is compiled in one of Typst's three syntactical modes, markup, math and code,
so inline math, whole doc comments and figures are one path rather than three features.

Two concerns are kept apart:
scanning a doc comment for fragments and their modes,
and rendering a fragment to HTML.

Rendering a fragment yields HTML together with the assets its page needs,
a stylesheet and a math font,
which are collected per page instead of emitted everywhere.

`typstdoc-core` holds all of this and is an ordinary library.
`typstdoc` is the proc macro over it, and `cargo-typstdoc` the command.

## Preamble

A `typstdoc.typ` at the crate root is prepended to every fragment,
so show rules, `#let` shorthands and imported notation hold across all doc comments:

```typst
#import "@local/dottyp:0.1.0": *
#let inner(a, b) = $lr(angle.l #a, #b angle.r)$
```

It is an ordinary Typst file, so `typst compile` checks it on its own.
A workspace is one body of notation,
so the command looks for it above the package as well and the nearest one holds.

## Fonts

MathML encodes the structure of an equation and leaves its appearance to the browser,
so Typst's output is the same whatever `#set text(font: ...)` says,
and setting a font in the preamble has no effect on it.
A page left to itself is set in whatever the browser resolves the `math` generic to.

Which font a page uses is therefore CSS.
`cargo typstdoc` hands rustdoc a stylesheet that names New Computer Modern Math
and carries the font itself, taken from the ones Typst embeds.
rustdoc copies that stylesheet to the root of the documentation
and links it from every page at that page's depth,
which is what lets one stylesheet reach pages lying at different depths.
The font travels inside it as a data URI,
since a stylesheet is the only thing a documentation build can hand rustdoc.

A build that renders through the macro rather than through the command
has no way to place that stylesheet, docs.rs among them.

## Relation to rustdoc

[RFC 3958](https://github.com/rust-lang/rfcs/pull/3958) adds LaTeX math to rustdoc
through `#![doc(enable(tex_math_dollars))]`, rendered to MathML by
[math-core](https://github.com/tmke8/math-core).
typstdoc follows that design with Typst as the source language.

That RFC rejected Typst for lacking an implementation-agnostic specification,
while its own future possibilities name `doc(syntax = "typst")` as an extension of the same grammar.
Mirroring its design keeps the question open.

typstdoc stays out of tree, since `doc(...)` is rustc's namespace,
so the attribute is its own and applies per module.

Inline and display math follow Typst's rule, spacing inside the delimiters,
rather than TeX's `$$`.

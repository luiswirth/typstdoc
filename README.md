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

Early, and nothing works yet.
The compile path is being built first; see the issues.

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
`typstdoc` is the proc macro over it.

## Preamble

A `typstdoc.typ` at the crate root is prepended to every fragment,
so show rules, `#let` shorthands and imported notation hold across all doc comments:

```typst
#import "@local/dottyp:0.1.0": *
#let inner(a, b) = $lr(angle.l #a, #b angle.r)$
```

It is an ordinary Typst file, so `typst compile` checks it on its own.

## Fonts

Typst embeds New Computer Modern and its math font, Libertinus Serif and DejaVu Sans Mono.
NewCM Sans and NewCM Sans Math ship with typstdoc.

Which font a page uses is CSS, configured under `[package.metadata.typstdoc]`.
MathML encodes the structure of an equation and leaves its appearance to the browser,
so Typst's output is the same whatever `#set text(font: ...)` says,
and setting a font in the preamble has no effect on it.

## Relation to rustdoc

[RFC 3958](https://github.com/rust-lang/rfcs/pull/3958) adds LaTeX math to rustdoc
through `#![doc(enable(tex_math_dollars))]`, rendered to MathML by
[math-core](https://github.com/tmke8/math-core).
typstdoc follows that design with Typst as the source language.

It stays out of tree, since `doc(...)` is rustc's namespace,
so the attribute is its own and applies per module:

```rust
#[cfg_attr(doc, typstdoc::typstdoc)]
mod hodge {}
```

Under `cfg_attr(doc, ...)` the macro is stripped in ordinary builds
and runs only when rustdoc does.

Inline and display math follow Typst's rule, spacing inside the delimiters,
rather than TeX's `$$`.

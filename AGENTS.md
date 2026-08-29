# typstdoc

Typst notation in Rust doc comments, compiled to MathML when rustdoc runs.
The README carries the design.

## Layout

- `crates/typstdoc-core/`:
  the library, doc string in and HTML out.
  Everything lives here.
- `crates/typstdoc/`:
  the proc macro over it, thin, because a proc-macro crate can export nothing else.
- `crates/cargo-typstdoc/`:
  the `cargo typstdoc` command, which stands in for rustdoc and renders a copy
  of a package's sources.

## Invariants

- **MathML carries no font information:**
  Typst's HTML export emits the same MathML whatever `#set text(font: ...)` says,
  and whatever fonts the world holds,
  so appearance is a CSS concern and never a Typst one.
- **A stylesheet is the only thing a documentation build hands rustdoc**,
  so the font it names travels inside it,
  and rustdoc is what links it from every page at that page's own depth.
- **`doc(...)` is rustc's namespace:**
  unknown keys inside it are an error, and a crate-level attribute macro needs unstable
  `custom_inner_attributes`, so the attribute is our own and applies per module.
- **The crate root is not a macro target:**
  an inner attribute macro is unstable, and the root is handed the injected prelude,
  which it cannot emit back.
  docs.rs builds on nightly only.
- **The macro runs only under rustdoc**, guarded by `#[cfg_attr(doc, ...)]` at the use site.
- **A rendered source keeps the line every item stood on**,
  since the pages rustdoc writes point into it.
- **Cargo is told a typstdoc build apart by the flags rustdoc is called with**,
  which is the only thing it watches besides the sources,
  so what a build rests on beyond them is stamped into them.
- **Fragments compile separately through one reused `World`:**
  never concatenated into one document and cut apart afterwards.
- **File resolution stays pluggable:**
  a docs.rs build is sandboxed and offline, so packages cannot be fetched while it runs.
- **Delimiter handling follows the pulldown-cmark math spec**,
  so `$` behaves as it does in rustdoc,
  except that spacing inside the delimiters is Typst's and never a second `$`.

## Verifying

Typst's HTML export is experimental and needs its feature flag:

    TYPST_FEATURES=html typst compile --format html eq.typ eq.html

`typst fonts --ignore-system-fonts` lists what the library embeds,
which is where the math font a page is set in comes from.

`cfg(doc)` distinguishes a documentation build from an ordinary one,
which is what keeps the macro out of `cargo build`.

What the macro does shows in the pages rustdoc writes rather than in `cargo test`,
so it is verified from a scratch crate depending on `crates/typstdoc`,
run with `CARGO_TARGET_DIR` pointed at this repository's target.

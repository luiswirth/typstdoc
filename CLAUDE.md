# typstdoc

Typst notation in Rust doc comments, compiled to MathML when rustdoc runs.
The README carries the design.

## Layout

- `crates/typstdoc-core/`:
  the library, doc string in and HTML out.
  Everything lives here.
- `crates/typstdoc/`:
  the proc macro over it, thin, because a proc-macro crate can export nothing else.

## Invariants

- **MathML carries no font information:**
  Typst's HTML export emits the same MathML whatever `#set text(font: ...)` says,
  so appearance is a CSS concern and never a Typst one.
- **`doc(...)` is rustc's namespace:**
  unknown keys inside it are an error, and a crate-level attribute macro needs unstable
  `custom_inner_attributes`, so the attribute is our own and applies per module.
- **The macro runs only under rustdoc**, guarded by `#[cfg_attr(doc, ...)]` at the use site.
- **Fragments compile separately through one reused `World`:**
  never concatenated into one document and cut apart afterwards.
- **File resolution stays pluggable:**
  a docs.rs build is sandboxed and offline, so packages cannot be fetched while it runs.
- **Delimiter handling follows the pulldown-cmark math spec**,
  so `$` behaves as it does in rustdoc.

## Verifying

Typst's HTML export is experimental and needs its feature flag:

    TYPST_FEATURES=html typst compile --format html eq.typ eq.html

`typst fonts --ignore-system-fonts` lists what the library embeds.
NewCM Sans and NewCM Sans Math are not among them and ship with typstdoc.

`cfg(doc)` distinguishes a documentation build from an ordinary one,
which is what keeps the macro out of `cargo build`.

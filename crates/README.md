# The Rust workspace

The front end is built: `cpl-diag`, `cpl-syntax`, `cpl-check` and `cpl-lsp`
have real code, and `cpl check`, `cpl run`, `cpl build` and `cpl lsp` all
go through one `cpl_check::Database`. `cpl-verify`, the verification
harness, is built too. `cpl-codegen`, `cpl-interp` and `cpl-fmt` are still
empty shells with one passing test. The split is a
starting point so that work can go on in parallel without two people
editing the same crate; change it when the code says it is wrong.

## One front end, reused per item

A file is lexed whole and cut into top-level items. Everything after that
is cached per item. Syntax is keyed by the item's text, with spans relative
to the item, so an item that only moved is not parsed again. A check records
each top-level name it looked up and a fingerprint of what it saw, and is
reused while those fingerprints match. Take this file:

```
fn link(mut g: [V]List<(V, Int)>, r: Road):
    g[r.u].push((r.v, r.len))

for e in E:
    link(mut g, roads[e])
```

Editing the body of `link` re-parses and re-checks `link` alone: its
signature has the same fingerprint, so the `for` loop is not touched.
Adding a parameter to `link` re-checks `link` and the loop, and nothing
else in the file. `crates/cpl-check/src/db.rs` has the details, and
`cpl-check`'s tests count the work each kind of edit causes.

Broken code is the normal case. The lexer, parser and checker never fail,
and one root cause gives one diagnostic: after its first error in a
statement the parser stays quiet until the line ends, and the checker says
nothing about code that already has an error.

To try it:

```
$ cargo run -q -p cpl-cli -- check proto/tests/bad/wrong_domain.cp
$ cargo test --release -p cpl-lsp --test latency -- --nocapture
```

The second command prints the latency of a cold check, a check after one
keystroke, and the language server's diagnostics and hover, on 500-line and
5,000-line files, and fails if any is over budget (50 ms for a check,
100 ms for the server).

The checker covers names, the types of bindings, fns, structs, enums and
domains, strict domain indexing, call arguments, and `mut`. Borrows, value
ranges, generics and traits are not checked yet, and the standard library
is not loaded: the prelude is a table in `cpl-check/src/prelude.rs`, and a
file with a `use` gets no "unknown name" errors.

The pipeline, in the order the crates will depend on each other (add the
dependency when the code first needs it):

- **`cpl-diag`** is diagnostics: spans, stable error codes, rendering.
  Every other crate reports errors through it (`plan/30-compiler.md` 3.7).
- **`cpl-syntax`** is the lexer, the parser and the syntax tree. The
  parser recovers from errors so the editor works on half-written code
  (`plan/00-vision.md` G13, `plan/17-syntax.md`).
- **`cpl-check`** is name resolution, types, domains, borrows and value
  ranges (`plan/11` to `plan/16`, `plan/30-compiler.md` 3.4 and 3.5).
- **`cpl-codegen`** turns a checked program into one C++17 file (`plan/30-compiler.md` 3.3, `plan/70-judges.md`).
- **`cpl-interp`** runs a checked program directly for debug runs and
  records the trace (`plan/30-compiler.md` 3.1, `plan/50-debugging.md`).
- **`cpl-fmt`** is the formatter (`plan/45-dev-tools.md`).
- **`cpl-lsp`** is the language server (`plan/60-editor.md`).
- **`cpl-cli`** builds the `cpl` binary and owns the commands (`plan/40-tooling.md`,
  `plan/45-dev-tools.md`).

Beside the pipeline, **`cpl-verify`** is the verification harness: the
differential fuzzer under ASan and UBSan, the `sizeof` audit, the raw and
library-stripped line counts and the latency benchmark
(`plan/80-validation.md`, `crates/cpl-verify/README.md`). It tests
`proto/cpc/` today and the crates above as they land.

```text
$ cargo test            # must pass before every push to main
$ cargo run -p cpl-cli -- --version
cpl 0.0.0
```

Not Rust: the VS Code extension is in `editors/vscode/`. Not here yet: the
standard library and the sample local library (`.cp` source), the trace
viewer and the docs site. Each gets its own top-level directory when it
starts.

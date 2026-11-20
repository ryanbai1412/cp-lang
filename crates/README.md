# The Rust workspace

Every crate here except `cpl-verify` is an empty shell with one passing
test. The split is a
starting point so that work can go on in parallel without two people
editing the same crate; change it when the code says it is wrong.

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

Not here yet, and not Rust: the standard library and the sample local
library (`.cp` source), the VS Code extension, the trace viewer and the
docs site. Each gets its own top-level directory when it starts.

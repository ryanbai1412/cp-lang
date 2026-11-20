# cp-lang

A competitive-programming language that compiles to C++17, and its
toolchain. The command is `cpl` (a working name) and source files end in
`.cp`.

- `plan/`: the source of truth for what we're building. Start with
  `plan/agent-context.md`, then `plan/README.md` and `plan/00-vision.md`.
- `decisions.md`: every language decision, its status and its reason.
- `research/`: the evidence behind the decisions (corpus mining and C++
  benchmarks).
- `Cargo.toml`, `crates/`: the Rust compiler and tools. An empty skeleton
  for now; `crates/README.md` says which crate owns what.
- `editors/vscode/`: the VS Code extension: TextMate grammar, the client
  for `cpl lsp`, run lenses and inline overlays.
- `proto/`: the Python prototype compiler (partial; see
  `plan/agent-context.md`). Kept for trying design changes quickly.
- `stdlib/`: the standard library, written in CP Lang, with its tests.
  `stdlib/README.md` is the tour.
- `examples/local-lib/`: a sample personal contest library built on it.
- `corpus/`, `subs/`: Ryan's accepted submissions, used for ports,
  benchmarks and design mining.
- `archive/`: superseded design docs.
- `site/`: the CP Lang documentation site and offline example playground. See
  [the site README](site/README.md) to build and preview it locally.

```text
$ cargo test                          # must pass before every push
$ cargo run -p cpl-cli -- --version
cpl 0.0.0
$ python3 proto/cp.py test stdlib examples/local-lib   # library tests
```

Work goes straight to `main`; keep it green.

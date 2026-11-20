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
- `proto/`: the Python prototype compiler (partial; see
  `plan/agent-context.md`). Kept for trying design changes quickly.
- `corpus/`, `subs/`: Ryan's accepted submissions, used for ports,
  benchmarks and design mining.
- `archive/`: superseded design docs.

```text
$ cargo test                          # must pass before every push
$ cargo run -p cpl-cli -- --version
cpl 0.0.0
```

Work goes straight to `main`; keep it green.

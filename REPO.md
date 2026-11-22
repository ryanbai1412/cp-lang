# REPO.md

Where everything lives, and where new things go. If you add a top-level
directory or move one, update this file in the same commit.

| Path | Holds |
|---|---|
| `AGENTS.md` | Start here: mission, goals, rules, reading order, commit convention |
| `README.md` | For people: what CP Lang is, install, quick start |
| `mise.toml`, `rust-toolchain.toml`, `justfile` | Unified tool version pins and task runner (`mise install && just check`) |
| `pyproject.toml`, `uv.lock` | Python dependencies for site, scoreboard, conformance, and corpus scripts |
| `package.json`, `pnpm-workspace.yaml` | JS/TS workspace covering `web/` and `editors/vscode/` |
| `LICENSE-MIT`, `LICENSE-APACHE` | The dual license: MIT or Apache 2.0, at your option |
| `spec/` | The normative 0.3 language: `core/`, `lib/`, `tools/`, `std/` stubs, `diagnostics.md`, `conformance/`, `delta/` (gaps, the hill-climb ledger `hillclimb.md`, dream comparisons), `review/`, gold dreams; `spec/README.md` is the index and `spec/STATUS.md` the current state |
| `decisions.md` | Every design call, its reason and how to reverse it; newest last |
| `plan/` | How we build it: `ARCHITECTURE.md`, `OPERATING.md` (settled decisions, the loop), `DOMAINS.md` (every area: a readable map on top, the detailed audit for agents below), `PORTING.md`, `agent-context.md` |
| `crates/` | The compiler and toolchain in Rust, one crate per component; `cpl-cli` is the `cpl` binary |
| `stdlib/` | The standard library, written in CP Lang |
| `corpus/` | Canonical corpus: `corpus/problems/<id>/` (`meta.json`, `port.cp`, `dream.cp`, `spec.cp`, `orig.cpp`, generators/checkers/interactors/tests), `corpus/lib/{scout,real}/`, `corpus/sets/`, `corpus/submissions/` (all raw accepted submissions), `corpus/VALIDATION.md`; `corpus/statements/` is a git-ignored statement cache |
| `research/` | Evidence and dated non-normative history: `PORTS.md`, `MIGRATION.md`, `frictions/`, `retros/`, `design-history/`, measurements |
| `docs/`, `site/`, `web/` | User documentation, the offline website/playground generator (`site/`), and web app (`web/`) |
| `editors/` | Editor integrations: VS Code, JetBrains, Neovim, Emacs, Helix (`editors/README.md`) |
| `bench/`, `examples/` | Benchmarks against C++; small runnable programs |
| `scripts/` | `check.sh` (the gate), hooks, statement fetching, conformance, scoreboard, corpus tooling |
| `tools/` | Standalone helper programs outside the main toolchain (e.g. `tools/trace/`) |

Not tracked: `target/` and `dist/` (build output), `corpus/statements/`.

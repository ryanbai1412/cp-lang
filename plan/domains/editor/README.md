# editor

Segment: C. Design, then build. Owns: `crates/cpl-lsp`, `editors/`. Worklog: [worklog.md](worklog.md).

## Covers
LSP and the editor packages (VS Code, JetBrains, Zed, Neovim, Emacs, Helix, tree-sitter).

## Improve
All requests on v0.3 syntax; latency; publish VS Code + Zed (marketplace) when release is ready.

## Success criteria
Every LSP request has fixture tests on v0.3; p95 latency under the budgets in the editor resume (diagnostics < 500 ms on the largest port); every editor setup tested in the gate; packages attached to each release.

## Ryan's role
Reviews the LSP feature list and which editors are tier 1.

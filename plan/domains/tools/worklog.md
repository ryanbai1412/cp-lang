# tools worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Note from debugger: `crates/cpl-cli/src/trace.rs` fills the viewer's new `trace-version` element with `VERSION`, and the versus note (also in `crates/cpl-wasm/src/lib.rs`) says "its brute" instead of `@brute(main)`. Revert that commit to undo.
- Request from diagnostics: `crates/cpl-lsp/src/server.rs:1832` tags overflow hints with `cpl_diag::Code("overflow")`, which is not a catalogue code, so `cpl explain` and editors cannot look it up. Suggest `cpl_diag::codes::OVERFLOW` (R0202, the trap the hint predicts) or a catalogued W code; diagnostics will add a W code if you prefer that.
- Review packet: README, PLAN, metrics.toml, ARCHITECTURE, design/overview.md, measure.py; questions tools-1..3 in plan/QUESTIONS.md. Found: spec/CLI 14 command mismatches; LSP latency test red at 5,000 lines in release (not in the gate); VS Code unit test 36/37 (stale prelude table, fixed on devin/wip-editor-lsp).
- Requested from compiler: a representation query for explain and hover (see compiler worklog).
- Lane opened.

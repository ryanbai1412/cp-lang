# tools worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Note from debugger: `crates/cpl-cli/src/trace.rs` fills the viewer's new `trace-version` element with `VERSION`, and the versus note (also in `crates/cpl-wasm/src/lib.rs`) says "its brute" instead of `@brute(main)`. Revert that commit to undo.
- Request from diagnostics: `crates/cpl-lsp/src/server.rs:1832` tags overflow hints with `cpl_diag::Code("overflow")`, which is not a catalogue code, so `cpl explain` and editors cannot look it up. Suggest `cpl_diag::codes::OVERFLOW` (R0202, the trap the hint predicts) or a catalogued W code; diagnostics will add a W code if you prefer that.
- Request from tech-debt (cold review round 1 of the packet, e4d04c25; rubric right home 3, one way 3, local next change 3, legible 4, finished migration 3). Paths and line counts check out (`wc -l crates/cpl-cli/src/*.rs`), with two fixes: the LSP's name generator is `crates/cpl-lsp/build.rs`, not under `src/`; `crates/cpl-cli/tests/` has 43 entries, not 40.
  1. Debt 2, one command table: one registry in `cpl-cli` (name, summary, flags, `--json`, exit codes) that drives help, dispatch and a `cpl help --json` dump. `web/scripts/gen-tools.mjs` reads the dump instead of scraping help text, and the spec-vs-binary check (debt 1) becomes a test over the same table. That gives one mechanism in place of three lists.
  2. Debt 4, `cpl explain` vs hover: the representation query belongs in the shared core (`plan/ARCHITECTURE.md` B3), with explain and hover both consumers; ask compiler for it rather than growing `explain_file`.
  3. Debt 6, debugger files in `cpl-cli`: move them once debugger's `cpl-trace` exists, as a pure move agreed by both lanes; it is not tech-debt's to do (Q2).
  4. Debt 7, other lanes' tests in `crates/cpl-cli/tests/`: list each file's owning domain in `plan/DOMAINS.md` (or a first-line `// domain:` comment) so a red test has an owner without moving it.
- Review packet: README, PLAN, metrics.toml, ARCHITECTURE, design/overview.md, measure.py; questions tools-1..3 in plan/QUESTIONS.md. Found: spec/CLI 14 command mismatches; LSP latency test red at 5,000 lines in release (not in the gate); VS Code unit test 36/37 (stale prelude table, fixed on devin/wip-editor-lsp).
- Requested from compiler: a representation query for explain and hover (see compiler worklog).
- Lane opened.

# compiler worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Landed `devin/wip-split-vm` (rebased): `crates/cpl-interp/src/vm.rs` (15367 lines) moved into `vm/` (14 files, largest 1771), no behaviour change; `cpl-interp` tests 64 pass; owned files over the cap 14 -> 13. Other lanes: rebase on it.
- Review packet: README, PLAN, metrics.toml, ARCHITECTURE (commit `plan(compiler): review packet`). Measured on `4b8f504f`: conformance 1617/1711 (47 causes), diff 1242/1336 (94 skip), owned-crate tests 429 pass. Filed 2 questions in `plan/QUESTIONS.md` and backed tech-debt's builtin-floor question. Next (phase 2, on go): land `devin/wip-split-vm`, `devin/wip-conformance-queue`, `devin/wip-product-domain`.
- Reply to diagnostics: no objection to switching the raw `cpl_diag::Code("E…")` literals to `codes::` constants; send it.
- Lane opened.
- Request from diagnostics: 11 error sites build codes as raw `cpl_diag::Code("E…")` literals (`crates/cpl-check/src/check/{expr,stmt,call}.rs`, `crates/cpl-syntax/src/parser/expr.rs:1086`), so the catalogue test does not see them and 7 are marked *(0.3)* though emitted. Diagnostics will send a mechanical commit switching them to `codes::` constants unless you object.
- Request from lint-fmt: a fact table in `cpl-check` (value range per expression, proven indexes, domain and array sizes, loop trip counts, call cost class) that `cpl-lint` and `cpl-lsp` read, so `crates/cpl-lint/src/rules/bounds.rs` and the `Proven*` passes can be deleted. lint-fmt will add contract tests first.
- Request from lint-fmt: `cpl-check` building for wasm32, so the playground gets the checker-backed lints (3 `target_arch` gates in `crates/cpl-lint`).

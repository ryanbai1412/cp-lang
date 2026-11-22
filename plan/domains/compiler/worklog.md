# compiler worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Lane opened.
- Request from diagnostics: 11 error sites build codes as raw `cpl_diag::Code("E…")` literals (`crates/cpl-check/src/check/{expr,stmt,call}.rs`, `crates/cpl-syntax/src/parser/expr.rs:1086`), so the catalogue test does not see them and 7 are marked *(0.3)* though emitted. Diagnostics will send a mechanical commit switching them to `codes::` constants unless you object.

# CP Lang tech debt, measured

Measured on the v0.2 deletion branch (main plus the deletion), 2026-11-22. Each item has its number, why it matters, and the fix.

## A. Leftover from v0.2 (finished in the deletion land and its follow-up)

| # | Debt | Size | Fix |
|---|---|---|---|
| A1 | Dead `if is_v03()` branches (the check is now a constant `true`) | 92 call sites in 11 files | Inline the v0.3 arm, delete the v0.2 arm and `edition.rs` |
| A2 | v0.2 corpus ports `corpus/problems/*/port.cp` (header says "CP Lang 0.2.2") | 877 files; 10 test/script files still read them | Delete; point tests at the 100 v0.3 `spec.cp` |
| A3 | Differential programs still in v0.2 syntax | 126 in `tests/pending/`, out of every sweep, with a check that the directory only shrinks | A child ports them back into `tests/programs/` |
| A4 | Legacy `site/` and old cheat sheet | 98 files, -12k lines | Already deleted on the site branch; merge it, restore the WASM size cap and browser smoke in `check.sh` |
| A5 | `decisions.md` summary rows contradicting the spec | 2 found (2.2, 2.9), fixed locally | Generate the summary from the decision entries so it can't drift |

## B. Stdlib privilege in the compiler (Ryan: "as little of stdlib in the compiler as possible")

| # | Debt | Size | Fix |
|---|---|---|---|
| B1 | Builtins matched by name in lowering (`lower/call.rs`) | 77 string-matched arms | Move each into `spec/std` as CP Lang, or into a declared `@intrinsic` with an explicit core-feature entry |
| B2 | Stdlib type names in IR, interpreter and codegen (`"Map"`, `"Heap"`, `"Prefix"`, ...) | 53 references | Same; the core may know `List`/`Map`/`Set` only as declared intrinsics |
| B3 | Stdlib names hard-coded in the linter | 7 names, some of removed APIs | `@lint` declarations in the stdlib (see `plan/LINTER.md`) |

## C. Unfinished stdlib

| # | Debt | Size | Fix |
|---|---|---|---|
| C1 | Public stdlib functions whose body is `todo` | 185 (graph 37, flow 19, bag 13, geometry 11, ...) | Fill in, conformance case per function; these are what contestants hit as "not implemented" |
| C2 | Private and test stubs `= todo` | 105 | Fill in or delete |
| C3 | Known blockers in `spec/std/GAPS.md` | 96 lines; `@cache` fields don't parse (DSU, `Comb`) | `@cache` is in progress with a child |

## D. Architecture shape

| # | Debt | Size | Fix |
|---|---|---|---|
| D1 | Very large files | `lower/call.rs` 13.3k, `interp/vm.rs` 12.1k, `lower/stmt.rs` 6.5k, `lower/expr.rs` 6.1k, `lower/mod.rs` 6.0k, `interp/lib.rs` 5.5k, `lint/rules/taste.rs` 4.5k lines | Split by concern as B1 drains them; most of `call.rs` is B1 |
| D2 | Two range/bound analyses: checker, and the linter's own | ~3k lines in `cpl-lint` (`bounds.rs`, `rules/mod.rs` proven facts) | One fact table in `cpl-check`; the LSP stops reading inlay hints from the linter |
| D3 | Browser has fewer lints than the CLI | the semantic half of `cpl-lint` is compiled out on wasm | Build the checker for wasm |
| D4 | Config spelling drift | the code uses `cp.toml` and `@allow`; the decisions say `cpl.toml` and `with lint(...)` | Rename; one spelling |

## E. Tests and tooling

| # | Debt | Size | Fix |
|---|---|---|---|
| E1 | Failing conformance cases | 68 of 1,624 at the last report (the child keeps adding cases) | Hill-climb; never delete a case |
| E2 | `#[ignore]` tests | 21 | Each either fixed or tied to a named conformance case |
| E3 | Recovery test below its bar (first-error accuracy < 99.5%, fix-its 1,525/4,081 < 88%) | 1 test, red before the v0.2 work | Own it in diagnostics |
| E4 | Script sprawl | 26 Python + 6 shell scripts beside `just` + `mise` | Each becomes a `just` recipe or moves into `cpl` |
| E5 | Hand-written JSON | `cpl-json` exists, but `cpl-cli` still has its own `json_escape` | Use `cpl-json` everywhere |

## F. Docs and history volume

| # | Debt | Size | Fix |
|---|---|---|---|
| F1 | `research/` | 1,003 files | Keep what a decision cites; delete the rest (git has it) |
| F2 | `decisions.md` | 5,650 lines | Keep as a log; generate the summary table and a "current design" index from it |

## Overall

Rust is 191k lines across 19 crates. The biggest real debts are **B** (stdlib in the compiler) and **C1** (185 public stdlib functions not implemented), because contestants feel both. **A** is cleanup that finishes this week. **D2** and **D3** gate the linter work.

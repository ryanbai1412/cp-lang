# lint-fmt worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- lint: L0506 `repeated_literal` no longer counts bounds in `read<T>()` type arguments or in struct and enum field types (the record's near-miss now covers both). Corpus `spec.cp` warnings 50 -> 43 (7 false positives of 8 L0506 hits; the one left, dmopc21c4p2, is real); `spec/std` 293 -> 287; `dream.cp` 83 -> 79 (`target/release/cpl lint <glob>` before and after). The other 43 corpus warnings are real by their lints' decisions: request in the idioms worklog.
- From compiler: `turn` is now in the prelude, so `spec/lints/redundant_prelude_import/near_miss.cp` imports `same_dir` instead (still not a prelude name); the record test passes again. Trivial unblock, same intent.
- lint: L0208 `redundant_prelude_import` no longer fires on a file of nothing but `use`s: `spec/std/prelude.cp` is the prelude's own list and drew 33 warnings. Test `the_prelude_file_is_not_its_own_redundant_import`. `cpl lint` totals (loop over `git ls-files`, release `cpl`): `spec/std/*.cp` 287 -> 254; corpus `spec.cp` 43 and `dream.cp` 79 unchanged.
- Re tech-debt round 1: `src/rules/literals.rs` (L0506, out of `taste.rs`) is in `rules/`, not `rules/taste/`, because L0506 is not a taste matcher; wasm gate count fixed to 8 in ARCHITECTURE.md.
- Landed 80374b28, lint: L0506 `repeated_literal` no longer counts bounds in `read<T>()` type arguments or in struct and enum field types (the record's near-miss now covers both). Corpus `spec.cp` warnings 50 -> 43 (7 false positives of 8 L0506 hits; the one left, dmopc21c4p2, is real); `spec/std` 293 -> 287; `dream.cp` 83 -> 79 (`target/release/cpl lint <glob>` before and after). The other 43 corpus warnings are real by their lints' decisions: request in the idioms worklog.
- Landed `devin/wip-idioms-lint` `dbc0692f`, rebased: every lint's explanation, trigger, fix and new near-miss live in `spec/lints/<name>/` (38 records) and `crates/cpl-lint` reads them with `include_str!`; tests `every_lint_record_fires_on_before_only` and `every_lint_record_directory_belongs_to_a_lint`. `cargo test -q -p cpl-lint`: 88 passed. Not landed from that branch: `6a895e56` and `c272ead4` (shared with `devin/idioms-docs`): `spec/idioms/*.md` is the idioms domain's (request in its worklog); `plan/LINTER.md` is superseded by `design/overview.md`; `plan/TECH-DEBT.md` is the tech-debt domain's call.
- From diagnostics: regression test that `cpl-fmt` keeps `x !in xs` tight (main's spacing already does), so the parser still recovers it as `not in` with its fix-it instead of reading `x! in xs`; test `keeps_a_bang_in_typo_one_operator` in `crates/cpl-fmt/src/tests.rs`. Exposed by `formats_the_whole_corpus` on `corpus/problems/cf2053i1/dream.cp`.
- Landed `devin/wip-fmt` (rebased, 4 commits; its decision renumbered 214 -> 215): c spellings stay tight, which fixes the one meaning change (`crates/cpl-cli/tests/fixes/c_operators.cp`; test `keeps_the_c_spellings_the_parser_explains_tight`), and the fmt gate is in `scripts/check.sh --fast`. `python3 scripts/fmt_check.py --cpl target/release/cpl`: 4443 files, 521 changed by cpl fmt, 0 not idempotent, 0 change meaning, 0 formatter bugs. The bulk reformat waits on Ryan (QUESTIONS.md).
- Request from tech-debt (cold review round 1 of the packet, 355d2592; rubric right home 3, one way 2, local next change 3, legible 4, finished migration 3):
  1. Boundary: the `Proven*` API and `rules/bounds.rs` are the only reason for the shipped edge `cpl-lint -> cpl-prover`, the one layer violation in the crate graph (`python3 scripts/debt-ratchet.py --list layer_violations`). Agree with moving them to `cpl-check` in M3; make "lint depends on `cpl-syntax`, `cpl-diag`, `cpl-check` only, on every target" the exit, which also ends the browser/CLI divergence.
  2. "3 `target_arch = "wasm32"` gates" undercounts: `git grep -c 'target_arch = "wasm32"' -- crates/cpl-lint` gives 8 (Cargo.toml 1, `src/lib.rs` 2, `src/rules/bounds.rs` 5).
  3. `src/rules/taste.rs` (4,493) and `src/rules/mod.rs` (2,144) are on the file-size allowlist, which can now only shrink. A new L05xx matcher should go in its own file under `rules/taste/`, not grow `taste.rs`.
  4. `scripts/fmt_check.py` landed and `scripts/check.sh:196` runs it by its shebang, so the shrink-only `direct_python` counter is unchanged; keep new gate scripts that way (or `uv run`), never `python3 <script>`.
- Phase 1 review packet: README, PLAN, metrics.toml, ARCHITECTURE,
  design/overview. Baseline (metrics.toml): 38 lints, 0 records on main;
  corpus `spec.cp` 50 warnings, `spec/std` 293; fmt changes 552 of 4,443
  tracked `.cp` files, 0 not idempotent, 1 meaning change. Next: phase 2
  starts by landing `devin/wip-fmt` (PLAN.md, "Phase 2, first").
- Requests filed: compiler (fact table, checker in wasm), idioms (who lands `spec/idioms/` from `devin/idioms-docs`).
- Lane opened.

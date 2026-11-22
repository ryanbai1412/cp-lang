# lint-fmt worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Landed `devin/wip-idioms-lint` `dbc0692f`, rebased: every lint's explanation, trigger, fix and new near-miss live in `spec/lints/<name>/` (38 records) and `crates/cpl-lint` reads them with `include_str!`; tests `every_lint_record_fires_on_before_only` and `every_lint_record_directory_belongs_to_a_lint`. `cargo test -q -p cpl-lint`: 88 passed. Not landed from that branch: `6a895e56` and `c272ead4` (shared with `devin/idioms-docs`): `spec/idioms/*.md` is the idioms domain's (request in its worklog); `plan/LINTER.md` is superseded by `design/overview.md`; `plan/TECH-DEBT.md` is the tech-debt domain's call.
- From diagnostics: regression test that `cpl-fmt` keeps `x !in xs` tight (main's spacing already does), so the parser still recovers it as `not in` with its fix-it instead of reading `x! in xs`; test `keeps_a_bang_in_typo_one_operator` in `crates/cpl-fmt/src/tests.rs`. Exposed by `formats_the_whole_corpus` on `corpus/problems/cf2053i1/dream.cp`.
- Landed `devin/wip-fmt` (rebased, 4 commits; its decision renumbered 214 -> 215): c spellings stay tight, which fixes the one meaning change (`crates/cpl-cli/tests/fixes/c_operators.cp`; test `keeps_the_c_spellings_the_parser_explains_tight`), and the fmt gate is in `scripts/check.sh --fast`. `python3 scripts/fmt_check.py --cpl target/release/cpl`: 4443 files, 521 changed by cpl fmt, 0 not idempotent, 0 change meaning, 0 formatter bugs. The bulk reformat waits on Ryan (QUESTIONS.md).
- Phase 1 review packet: README, PLAN, metrics.toml, ARCHITECTURE,
  design/overview. Baseline (metrics.toml): 38 lints, 0 records on main;
  corpus `spec.cp` 50 warnings, `spec/std` 293; fmt changes 552 of 4,443
  tracked `.cp` files, 0 not idempotent, 1 meaning change. Next: phase 2
  starts by landing `devin/wip-fmt` (PLAN.md, "Phase 2, first").
- Requests filed: compiler (fact table, checker in wasm), idioms (who lands `spec/idioms/` from `devin/idioms-docs`).
- Lane opened.

# infra worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- CI red, first cause: emitted C++ used `std::exchange` without `<utility>` (6 of 1116 tests on GitHub runners, run 45067797421). Trivial unblock in `crates/cpl-codegen/src/emit.rs` with a codegen test, noted in perf's worklog. Evidence: `cargo nextest run -p cpl-codegen` (89 passed).
- Review packet: README, PLAN, metrics.toml, ARCHITECTURE, design/gate-tiers.md; 3 questions in plan/QUESTIONS.md. Measured: warm gate 3 s unchanged, 77 s after a one-line `cpl-fmt` edit (nextest 57 s, 1116 tests), 101 s after a fresh build; CI 0/6 green on main; 11 superseded `devin/*` branches.
- Request for perf (filed in phase 2): emitted C++ uses `std::exchange` without `<utility>` (CI run 45067797421).
- Note from lint-fmt: added the `cpl fmt is idempotent and keeps meaning on every .cp file` step to `scripts/check.sh` (runs `scripts/fmt_check.py` directly, so `direct_python` stays 13, beside nextest, cached as stage `fmt_corpus` in `scripts/check_cache.py`; about 6 s on 8 cores, cold), plus `scripts/test_fmt_check.py` in the script unit tests. Revert that commit to drop it.
- Request from site: the docs-only eval cannot run. `.github/workflows/writable-from-docs.yml` calls `scripts/fetch-statement.py`, deleted in `e4fc2a14`, and `corpus/statements/` is absent (`python3 scripts/writable-from-docs.py --manifest research/writable-from-docs-set.json` finds 0 of 55 statements). Please restore a working statement step (with idioms, who own the statements) and point the bundle at the served `/llms-full.txt` so the eval reads what cp-lang.org serves.
- Request from site: run `scripts/site-check.sh` (wasm build and size cap, site build, snippet gate, link check, browser smoke) in `.github/workflows/site.yml` before deploy; it needs pnpm and a Playwright Chromium.
- Note from site: to wire `scripts/site-check.sh`, site touched infra files directly: `scripts/check.sh` (`--full` runs it), `scripts/check_cache.py` (`site_smoke` stage), `scripts/test_check_cache.py`, `justfile` (`just site`).
- Note from site: `python3 -m unittest scripts.test_check_cache.CheckCacheTest.test_land_carries_queued_lanes_in_one_gated_train` fails on `origin/main` (a leftover `refs/locks/queue/*-rider` ref).
- Note from idioms: `scripts/ports-page.py` no longer imports the deleted `scripts/scoreboard.py`; it loads port discovery and the CP tokenizer from `scripts/review-check.py` and keeps the C++ tokenizer, library stripping and friction parsing it needs. New `--tokens` prints the token-ratio summary. This closes the idioms token-count request.
- Note from idioms: the skill landing adds `scripts/skill-gen.py` + `scripts/test_skill_gen.py` and a "bundled skill" step to `scripts/check.sh` (via `uv run python`, so `direct_python` stays 13). Request: `scripts/ports-page.py` imports `scoreboard`, deleted in ea2b88ff, so the token-ratio metric cannot run; idioms will restore a token counter unless infra prefers another home.
- Lane opened.

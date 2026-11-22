# idioms worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Request from tech-debt (cold review round 1 of the packet, 397e6655; rubric right home 3, one way 3, local next change 4, legible 4, finished migration 2). Paths and counts check out (1160 problem dirs, 305 in `corpus/sets/ecnerwala.txt`, `cpl-judge` has no dependencies).
  1. Boundary with the v0.2 cleanup: the 877 `port.cp` files are the tech-debt `v02_port_files` counter (shrink-only), but 305 of them are your tracked ports and your migration sources. Proposal: idioms decides which `port.cp` stay (the 305 until each has a v0.3 `spec.cp`); tech-debt deletes only the others and never touches the 305. Reply here if you want a different split.
  2. `corpus/cp.toml` `lib = "../examples/local-lib"` points at a deleted path: delete the line (trivial, yours) rather than list it as debt.
  3. `crates/cpl-cli/tests/judges.rs` parsing `research/scoreboard.json` by string slicing is tools' code: file it in `plan/domains/tools/worklog.md` with the `cpl-json` fix.
  4. Layout: keep the review, corpus and docs-eval scripts in `scripts/` (one home for gate scripts, owned per file in `plan/DOMAINS.md`) rather than moving them under `corpus/`. When you change them, call them through `uv run`, not `python3` (`review-check.py` is one of the 13 `direct_python` calls the ratchet counts in `scripts/check.sh`).
  5. `cpl-conform` owner (your request to the coordinator): tech-debt recommends compiler owns the crate (it runs the conformance suite over the shared core) and idioms owns the corpus data and scoreboard it reads.

- Landed: token ratio measurable again. `scripts/ports-page.py` drops the `scoreboard` import (helpers from `review-check.py` + the C++ tokenizer/stripper inlined); `--tokens` gives port.cp 0.52x median, max 1.10x (only ucup5s05f over 1.0x), dream.cp 0.49x, max 0.97x, n=294 (11 originals strip to nothing, skipped). research/PORTS.md regenerated (paths now corpus/problems).
- Landed: `devin/skill-eval-1` as 3ee895ac (the cold run report, 20 solutions, 5 repros under `research/skill-eval/`; historical, run on the skill before its field-listing fix). Next: a fresh blind run on the known 20 as the regression baseline.
- Landed: `devin/skill-v2` as af906f8b (3 commits squashed: skill, generator, `skill-gen.py --check` in the fast gate via `uv run python`; 40 generated stdlib pages refreshed). Evidence: `skill-check: ok (48 files, 75 cp blocks verified)`; fast gate green. `devin/cp-lang-skill` is superseded (`git cherry` shows its only commit inside skill-v2); not landed.
- Request from lint-fmt: `devin/idioms-docs` carries `spec/idioms/lints.md` and `spec/idioms/principles.md` (shared commits with `devin/wip-idioms-lint`). Proposal: idioms lands those two files; lint-fmt lands `spec/lints/` records and the engine. Each catalog entry that should become a lint needs a before/after/near-miss triple.
- Review packet: README, PLAN, metrics.toml, ARCHITECTURE; 3 questions in plan/QUESTIONS.md. Evidence: every value in metrics.toml from a run on this tip.
- Request (infra): `scripts/ports-page.py` fails (imports `scoreboard`, deleted in ea2b88ff), so token ratio is unmeasurable; want a token-count helper and `skill-gen.py --check` in the fast gate.
- Request (coordinator): DOMAINS.md names no owner for `crates/cpl-conform`; idioms owns the corpus suite and scoreboard data only.
- Request from site: the docs-only eval has no statements. `research/writable-from-docs-set.json` names 55 problems, but `corpus/statements/` is absent and `scripts/fetch-statement.py` was deleted in `e4fc2a14`. Please commit the 55 statements (or restore the fetcher) so `python3 scripts/writable-from-docs.py --manifest research/writable-from-docs-set.json` runs; infra has the matching CI request.
- Lane opened.

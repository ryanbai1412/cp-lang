# idioms worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Request from lint-fmt: `devin/idioms-docs` carries `spec/idioms/lints.md` and `spec/idioms/principles.md` (shared commits with `devin/wip-idioms-lint`). Proposal: idioms lands those two files; lint-fmt lands `spec/lints/` records and the engine. Each catalog entry that should become a lint needs a before/after/near-miss triple.
- Review packet: README, PLAN, metrics.toml, ARCHITECTURE; 3 questions in plan/QUESTIONS.md. Evidence: every value in metrics.toml from a run on this tip.
- Request (infra): `scripts/ports-page.py` fails (imports `scoreboard`, deleted in ea2b88ff), so token ratio is unmeasurable; want a token-count helper and `skill-gen.py --check` in the fast gate.
- Request (coordinator): DOMAINS.md names no owner for `crates/cpl-conform`; idioms owns the corpus suite and scoreboard data only.
- Lane opened.

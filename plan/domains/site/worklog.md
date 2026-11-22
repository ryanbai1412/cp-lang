# site worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Review packet: README, PLAN, ARCHITECTURE, metrics.toml, screenshots/; three questions filed in `plan/QUESTIONS.md`; `web/scripts/links.mjs` (`pnpm --dir web links`) added to measure internal links.
- Evidence, all on main `4b8f504f`: `node web/scripts/gate.mjs` 197 snippets (runs 186, checks 4, spec 7); `scripts/build-wasm.sh` 1,761,196 B gzipped; `pnpm --filter cp-lang-web build` pass 28.9 s, 239 pages; typecheck pass, vitest 4/4; `web/tests/smoke.mjs` and `learn-smoke.mjs` pass, 0 page errors; `pnpm --dir web links` 1 broken target (`/docs/guides/`); search index 2,690,140 B gzipped; cp-lang.org 8/8 routes 200.
- Not measurable: docs-only eval (`corpus/statements/` absent, `scripts/fetch-statement.py` deleted in `e4fc2a14`, workflow still calls it).
- Next (phase 2): land `devin/v02-delete-site-followup`, fix `/docs/guides/`, links in the gate, UI pass.
- Requests to file in phase 2: infra, run `scripts/site-check.sh` in `.github/workflows/site.yml` and add per-push previews; idioms/infra, restore statements for the eval and point it at `/llms-full.txt`; compiler, the 11 `spec`/`checks` snippets from `web/.gate/report.md`.
- Lane opened.

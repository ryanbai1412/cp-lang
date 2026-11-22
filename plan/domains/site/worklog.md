# site worklog

Newest first: what landed (commit), evidence, next, blocked, requests from other lanes.

## 2026-11-22
- Landed `site: index reference headings and intros only in search`: `gzip -c web/out/api/search | wc -c` 2,690,140 -> 438,011 (raw 15,495,653 -> 2,526,692). The reference's per-entry text left the index (4,479 -> 2,295 docs under `/docs/ref/std`) and Orama's unused sort index is off. Reverse: return `structuredData` unchanged in `web/app/api/search/route.ts`. Evidence: `node web/tests/run.mjs smoke.mjs` -> `search finds a reference item and a guide heading`, `page errors: []`.
- Phase 2: landed `390f9623` `site: spell the whole-program brute @alt(program)` and `5936d337` `site: cap cpl.wasm's gzipped size and run the browser smoke in a cached stage` (cherry-picked from `devin/v02-delete-site-followup`; its other commits are the compiler's v0.2 deletion, not site's; this branch can be deleted). `scripts/site-check.sh` now also runs `pnpm --dir web links`. Dropped from the pick: edits to `spec/lib/rationale/README.md` and `crates/cpl-cli/tests/cases/cheatsheet/README.md` (not site paths).
- Landed `4b90f665` `site: link the reference pages' intro to the tour, not a missing index`: links 1 broken target -> 0.
- `devin/wip-site-plans`: `user-journey.md` moved to `design/user-journey.md` (Ryan agreed to it 2026-11-21); `site-plan.md` and `RESUME.md` are superseded by README, PLAN and ARCHITECTURE; the branch can be deleted.
- Evidence: `CDP=... scripts/site-check.sh --no-cache` -> `wasm-size: cpl.wasm is 5,784,283 bytes, 1,768,985 gzipped (cap 2,100,000)`, `gate: 197 snippets { runs: 186, spec: 7, checks: 4 } ok`, `links: 8726 internal links, 0 broken targets`, smoke `page errors: []`, `site-check: ok`.
- Requests filed: infra (eval CI step, site-check in CI, `test_check_cache` land-train test fails on main; note that site touched `scripts/check.sh`, `check_cache.py`, `test_check_cache.py`, `justfile`), idioms (55 eval statements).
- Review packet: README, PLAN, ARCHITECTURE, metrics.toml, screenshots/; three questions filed in `plan/QUESTIONS.md`; `web/scripts/links.mjs` (`pnpm --dir web links`) added to measure internal links.
- Evidence, all on main `4b8f504f`: `node web/scripts/gate.mjs` 197 snippets (runs 186, checks 4, spec 7); `scripts/build-wasm.sh` 1,761,196 B gzipped; `pnpm --filter cp-lang-web build` pass 28.9 s, 239 pages; typecheck pass, vitest 4/4; `web/tests/smoke.mjs` and `learn-smoke.mjs` pass, 0 page errors; `pnpm --dir web links` 1 broken target (`/docs/guides/`); search index 2,690,140 B gzipped; cp-lang.org 8/8 routes 200.
- Not measurable: docs-only eval (`corpus/statements/` absent, `scripts/fetch-statement.py` deleted in `e4fc2a14`, workflow still calls it).
- Next (phase 2): land `devin/v02-delete-site-followup`, fix `/docs/guides/`, links in the gate, UI pass.
- Requests to file in phase 2: infra, run `scripts/site-check.sh` in `.github/workflows/site.yml` and add per-push previews; idioms/infra, restore statements for the eval and point it at `/llms-full.txt`; compiler, the 11 `spec`/`checks` snippets from `web/.gate/report.md`.
- Lane opened.

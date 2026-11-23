# site plan

The detail behind [README.md](README.md). Dated 2026-11-22.

## Where it stands

The site works: build, gate, typecheck, unit tests and both browser smokes pass, cp-lang.org serves every route, and the landing runner, trace replay, stress shrink and Learn judging all work in a real browser (numbers in [metrics.toml](metrics.toml)). What is missing is mostly *proof*: the domain's own bar (the docs-only eval) cannot run, the site checks are not in CI, and nothing caps size. The remaining product work is consistency and depth.

## Workstreams, in order

1. **Land and gate (M1).** Rebase `devin/v02-delete-site-followup` (wasm cap, `scripts/site-check.sh`, smoke in a cached stage) onto main; drop what `git cherry` shows superseded; add `web/scripts/links.mjs` to the gate path; ask infra to run `scripts/site-check.sh` in `.github/workflows/site.yml` and to add per-push Worker previews. Then `devin/wip-site-plans`: move the still-true parts of `plan/site/` into this folder, drop the rest.
2. **Restore the bar (M2).** The eval needs statements and a fetcher. Ask idioms/infra (owners of the harness) to restore `scripts/fetch-statement.py` or commit the 55 statements, and to point the bundle at `/llms-full.txt` so the eval reads exactly what the site serves. Site's part: make `/llms-full.txt` complete and ordered for a model, then triage every failure: docs gap (fix here, add a guide example) or language gap (file with the owner).
3. **UI pass (M1).** One nav placement across `HomeLayout` and `DocsLayout`; landing tabs lead with programs that run; phone pass at 390 px on every route; light and dark. Screenshot every change into the worklog.
4. **Size (M1).** Search index from 2.7 MB to ≤ 500 KB gzipped: index the reference by symbol and summary instead of full text, or split it per section. Keep the wasm cap.
5. **Live guides (M2).** Fenced `cp` blocks with status `runs` render as a compact `Runner` (Run, Open in playground). Static otherwise, with the status badge.
6. **Learn depth (M3).** Done at the proposed size: chapters 4-8 (functions, records and match; strings and output; sorting and searching; the standard library; stress and trace) and 8 more original problems, each with reference, tests and a judged starter in the gate. Then, on Ryan's scope ("learn: through guides, through practice problems"), a graphs chapter and a DP chapter with two judged problems each: 10 chapters, 19 problems. Interval DP waits on `V.ranges` in the compiler.
7. **Spec-only snippets to zero (M3).** The 7 `spec` and 4 `checks` snippets wait on the compiler; list them in the compiler worklog with the gate report, flip each the day it runs.

## Why these milestones

- **M1 first** because every later round of Ryan's review rests on "main is green and what you see is what ships"; today a size regression or a broken link reaches cp-lang.org silently.
- **M2** is the mission bar. Without a fresh number every docs change is a guess; with it, docs work is a hill to climb.
- **M3** is the launch content. It depends on the compiler closing spec gaps, so it goes last and tracks their progress.

## Alternatives considered

- **Eval against the repo `spec/` instead of the served docs.** Rejected: it measures the spec, not the site; the cold reader sees cp-lang.org.
- **Server-side compile for share previews or slow devices.** Rejected: costs a service and a sandbox; the wasm worker already runs everything at 1.8 MB gzipped.
- **Drop Fumadocs search for an external index (Algolia, Pagefind).** Pagefind stays the fallback if trimming the Fumadocs index does not reach the target; no third-party service.
- **Hide `spec`/`checks` examples entirely.** Rejected for now: they show 0.3; the badge keeps them honest. Question 1 asks Ryan.

## Risks

- The eval depends on two other domains and a model key; if it stays down, M2 slips. Mitigation: a small local probe (P1, 3 problems) that site runs itself.
- wasm growth from compiler work can break the cap; the cap's failure message names the size and the owner.
- `prebuild` runs the full gate (about a minute with the release `cpl`); a slow gate slows every build. Cache by snippet hash if it passes 2 minutes.
- Generated reference shape is set by `cpl doc` (tools); layout fixes there may need tools changes.

## Phase 2, first moves

1. Rebase and land `devin/v02-delete-site-followup` through `scripts/land.sh`.
2. Fix the `/docs/guides/` target and wire `web/scripts/links.mjs` into the gate path.
3. File the eval and CI requests in the infra and idioms worklogs.
4. Start the UI pass with the nav and landing tabs, screenshots in the worklog.

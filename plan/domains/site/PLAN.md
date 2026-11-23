# site plan

The detail behind [README.md](README.md). Dated 2026-11-22.

## Where it stands

The site works: build, gate, typecheck, unit tests and both browser smokes pass, cp-lang.org serves every route, and the landing runner, trace replay, stress shrink and Learn judging all work in a real browser (numbers in [metrics.toml](metrics.toml)). What is missing is mostly *proof*: the domain's own bar (the docs-only eval) cannot run, the site checks are not in CI, and nothing caps size. The remaining product work is consistency and depth.

## Workstreams, in order

Ordered by [Milestone 1](../../product/MILESTONE-1.md): work that blocks steps 1-2 of its scenario (Understand, Try) or the way into step 3 (Install) comes first.

1. **30-minute intro (step 1). Landed.** One page, `/learn/intro`, built from `web/content/docs/guides/tour.mdx` and the cheat sheet: syntax, typed input, domains, optionals, the std, and testing, ≤ 4,500 words, every example a live runner. It ends at Practice and Install. The homepage's first call to action points to it.
2. **Language section (step 1, in depth; parallel to 1 and 3).** `/learn/language/`, 8 chapters: values, ownership and mutation (`mut`, aliasing, invalidation); domains, ranges and index spaces (range keys); types, widths and proofs; optionals; multitest state; checked operations and build profiles; stdlib design; the C++ and the judge model. Each one states its rules precisely in site prose that links the `spec/` section it follows (no copied spec text), has gate-run examples that run in place, and ends with "Why it's designed this way", citing `decisions.md` and the alternatives they rejected. The gate checks every cited spec heading and decision number, so a renamed section fails the build instead of drifting.
3. **Practice that teaches (step 2).** Pick at least 12 Div 2 A/B problems from the 21 that teach CPL's distinctive patterns. Give each a `hints` list (shown before acceptance, linking to the std docs or a guide). Lint live in the practice and playground editors through the wasm `lint` call tools landed (`25b19e064`).
4. **Homepage and install (steps 1, 3).** `/setup` landed (`30b229105`): edge on Linux, macOS and Windows; Neovim, Vim, VS Code and JetBrains in one step each. Next: `cpl setup neovim|vim` once edge ships it; the value proposition and 3 differentiators above the fold at 390 and 1280 px; Install in the header at every width; a P5 cold-read baseline.
5. **Docs examples all run (step 2).** The 7 `spec` snippets go to zero as the compiler lands them; each one left is badged and listed with its owner. Search finds every std item by name. The docs-only eval measures this once a model key or CI run exists.
6. **Keep green (standing).** Gate, links, phone pass, `/` page weight ≤ 800 KB with no wasm, search ≤ 500 KB, wasm cap.

Done before Milestone 1: land and gate (wasm cap, `scripts/site-check.sh`, link check); the UI pass (one nav, phone fit, landing tabs that run first); search 2.7 MB to 491 KB; live guide runners; Learn at 11 chapters and 21 problems; `cpl doc` HTML in the site's design; the trace viewer links back.

Deferred (milestone "what can wait"): interval DP and chapters past Div 2 C; trace viewer and `cpl doc` polish; 0.3-only landing tabs; Problems and Performance pages; size tuning beyond the caps.

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

## General fixes owed

Each row: the instance, the general fix, and who owns it.

- **Practice refs that our own lints flag** (5 of 21 refs; hints taught the flagged form). Made general in the site gate: every practice `ref.cp` must lint clean and every problem needs a hint that links to the docs, a chapter or a lint. Still owed (site): lint every runnable docs and Learn snippet the same way, with an allow marker for snippets that show the flagged form on purpose.
- **`loop_reduction` says "is one `min`"**, but `min(e for ...)` is `E0303` and `(e for ...).min()` does not parse; refs use `[e for ...].min()` and `filter(p).max()`. General fix: generator arguments for `min`/`max`, as for `sum` (compiler/stdlib), and help text that names a spelling that compiles (lint-fmt). Filed in lint-fmt's worklog.
- **Record patterns `Add(x:)` do not parse** (R02.7.6, decision 230); the values chapter uses positional `Add(x)`. General fix: field patterns in the parser (compiler). Filed in compiler's worklog.
- **Generic trap messages for slice keys (`w[0]`, R04b.1.10) and `!` on `[V]?Int` (R02.4.8)**; the chapters explain them in prose. General fix: key-aware and gap-naming trap messages (diagnostics). Filed in diagnostics' worklog.
- **Std doc examples that don't run** (286 of 2,767; shown static). Site makes each one a program and runs it at build time. General fix: `cpl doc --test` runs examples, not just parses them, so stdlib's own check fails on a broken one (tools/stdlib). Filed in stdlib's worklog.
- **`cpl doc` dropped `pub` from fn signatures** (488 of 730), and the site guessed visibility from the signature text. Fixed in the extractor. General fix owed (tools): the JSON carries visibility as a field, so no consumer parses signatures for it.
- **Hover shows intra-doc links as code, not links.** Hover has no module for a std name, so it can't build the cp-lang.org URL. General fix owed (tools/lsp): hover carries the std module of the hovered item, then links render as `https://cp-lang.org/docs/ref/std/<module>#<anchor>`.

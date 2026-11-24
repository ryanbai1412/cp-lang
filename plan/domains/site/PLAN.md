# site plan

The detail behind [README.md](README.md): what is open, in order, and what other lanes owe the site. Standing rules live in [handbook/product/site/](../../../handbook/product/site/README.md#standing-rules), not here. Dated 2026-11-24.

## Where it stands

cp-lang.org serves every route from `web/out`, with the compiler in the page (`cpl.wasm`, lazy std, under the 1,875,000 B gz cap). The prebuild is cached by content hash per stage (`web/scripts/prebuild/`), so `pnpm site:test <suite>` rebuilds only what changed and runs the named browser suites. Each suite runs against one shared Chromium and fails by name if it hangs, exits without a verdict, or outlives it (`web/tests/run.mjs`). Every LSP message goes through one queue in `web/lib/cpl.ts` (`lspTask`), tested against a fake worker in `web/lib/lsp.test.ts`. One header (`web/components/site-header.tsx`) serves both Fumadocs layouts. The a11y, search and completion browser gates fail on any finding; their known lists are deleted.

## Open work, in order

Ordered by [Milestone 1](../../../handbook/product/milestones.md): steps 1-2 of its scenario (Understand, Try) and the way into step 3 (Install) first.

1. **Run on every landing example.** 6 of the 8 syntax and library examples run. The other 2 wait on checker fixes (Core): `egoi23p4` (a bare generic struct `Semiring` as a parameter type), `btoi20p3` (`undoing` nested in `.minimal(...)`). Each runs as soon as its fix lands: rename `.spec.cp` to `.cp` and add its `.in` and `.out`.
2. **Std doc examples all run.** `web/scripts/doc-examples-failing.txt` lists the ones that don't, each a std or checker bug (Content, Core). Delete the file once it is empty; `skip_lists` counts it until then.
3. **Practice that teaches.** At least 12 Div 2 A/B problems with hints before acceptance (21 have hints) and live lints in the practice and playground editors.
4. **Homepage cold read.** Probe P5: a cold reader names 3 advantages over C++ after 3 minutes, at 390 and 1280 px.
5. **Milestone 2.** Learn to Div 1 C with at least 60 judged problems, a Language chapter per part of `spec/core/`, stress and trace in the playground, and the docs-only eval passing at least 50 of 55 from the site alone.

## General fixes owed by other lanes

- **Std examples run in `cpl doc --test`** (tools, stdlib): run, not just parse, so a broken example fails stdlib's own check before the site build.
- **Lint examples compile** (lint-fmt): `record_tests.rs` requires `before.cp` and `near_miss.cp` to pass `cpl check`; a lint whose pattern is now a compile error is retired.
- **Generator arguments for `min`/`max`** (compiler) and lint help that names a spelling that compiles (lint-fmt).
- **Record field patterns `Add(x:)`** (compiler, decision 230).
- **Key-aware trap messages** for slice keys and `!` on `[V]?Int` (diagnostics).
- **Visibility as a JSON field in `cpl doc`** (tools), so no consumer parses signatures.
- **Hover links std items** (lsp): hover carries the item's std module so intra-doc links become cp-lang.org URLs.

## Known debt

- The docs-only eval cannot run: `.github/workflows/writable-from-docs.yml` calls `scripts/fetch-statement.py`, deleted in `e4fc2a14`, and `corpus/statements/` is absent.
- Search index at 487 KB gz of its 500 KB cap; the next cut is per-entry term frequencies (about 140 KB gz).
- `page-weight` does not yet report prefetched payloads per link, so a heavy linked page can grow `/` unseen.
- `/learn/start` and `/learn/from/*`, drawn from code, are not rendered into `/llms-full.txt` from their pages' data.

Checkpoints go in [worklog.md](worklog.md).

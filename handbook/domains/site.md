# Domain: site

| | |
|---|---|
| Kind | Domain charter |
| Owner | site |
| Status | Active |
| Describes | `web/`, `scripts/site-check.sh` |

cp-lang.org is where most people first meet CP Lang. It teaches the language from a first program to the whole design, gives every tool (check, lint, run, trace, stress, explain, build) in the browser, and hosts the std reference. The site domain owns those pages, the playground and the editors embedded in every page.

## What good looks like

A Codeforces regular who has never heard of CP Lang lands on the home page, presses Run on the first program, and sees why it beats their C++ template within three minutes. An hour later they have read the first Learn chapters and solved a judged practice problem in the browser. In a contest they keep the std reference open in a tab: hovering `runs` in the playground shows its signature and what `by:` takes, Cmd+click opens its entry in a new tab, and the entry lists its parameters, its cost as math and an example that runs. Nothing they copy from any page fails to compile, and the C++ the browser produces is the C++ `cpl build` would have produced.

## How it fits together

The site is a static Next.js export under `web/`, rebuilt on every main commit from the compiler that ships with it.

- **Pages** are MDX under `web/content`: Learn (Read chapters and judged practice), guides, Language chapters, the cheat sheet, setup, error and lint pages.
- **Generated pages** come from the compiler at build time: the std reference from `cpl doc --format json`, lint and error pages from the registries, tool pages from `cpl --help`.
- **The gate** compiles and runs every program on every page and compares it with what the page claims. A page that is wrong fails the build.
- **Editors** are CodeMirror backed by the compiler built to WebAssembly. `cpl.wasm` holds checking, running, tracing and the same language server desktop editors use; `cpl-build.wasm`, with C++, Explain and Stress, loads on first use.

## How we know it is good

The build fails on a page whose program misbehaves, a contradicting fact, a broken internal link, an oversized search index or an oversized `cpl.wasm`. Browser tests check Run, Trace, share links, hover, completion, go to definition, inlay hints and new-tab links. The weakest areas today are the ones a contestant feels while writing: no check yet that the browser editor's answers match the language server's request by request, or that every std item hovers and completes. Accessibility has a browser check with a list of known violations that only shrinks. The quality register below lists every quality with the check behind it; rows marked `missing` are the work queue. The rules pages follow are in `handbook/guidelines/site.md`.

## Where to start

- **Adding a page or a practice problem:** read `handbook/guidelines/site.md`, add the MDX or the problem folder under `web/content`, and run `pnpm --dir web build`; the gate tells you what is wrong.
- **Changing how the std reference renders:** `web/scripts/gen-ref.mjs`.
- **Adding an editor feature:** add it to the language server (tools), then reach it from `web/components/editor/intelligence.ts` and add a case to `web/tests/intelligence.mjs`.

---

# Reference

## Scope

| Site owns | Not site's (owner) |
|---|---|
| `web/`: landing, Learn (Read chapters and judged practice), guides, Language chapters, cheat sheet, `/setup`, `/errors`, `/lints`, `/decisions` | What the language does: `spec/` (spec); std `///` doc text (stdlib) |
| Rendering the generated std reference (`web/scripts/gen-ref.mjs`) and its runnable examples | `cpl doc --format json`, `cpl --help`, the language server (tools) |
| The playground and embedded editors: CodeMirror mode, live diagnostics, hover, completion, Run, Trace, Stress, C++, Explain, share links (`web/components/`, `web/worker/`) | The wasm API in `crates/cpl-wasm` and its analyses (tools, debugger); the stress engine and trace viewer (debugger) |
| The browser build's split and size: editor-first `cpl.wasm`, lazy `cpl-build.wasm`, `web/scripts/wasm-size.mjs` | `scripts/build-wasm.sh`, CI workflows, deploying cp-lang.org (infra) |
| The snippet gate `web/scripts/gate.mjs` and the facts gate `web/scripts/facts.mjs` | The gate's place in `scripts/check.sh` and CI (infra) |
| The docs bundle models read: `/llms.txt`, `/llms-full.txt` | The docs-only eval harness `scripts/writable-from-docs.py` (idioms, infra) |
| Commands: `pnpm --dir web build`, `typecheck`, `test`, `gate`, `links`, `smoke`, `phone`, `page-weight`, `wasm-size` | `corpus/` (idioms), `bench/` data (perf), error registry `crates/cpl-diag/src/registry.rs` (diagnostics) |

## Interfaces

**Site provides**

| Interface | Used by |
|---|---|
| `web/out/` static export and `web/worker/wrangler.toml` | infra: deploy |
| `node web/scripts/gate.mjs` exit code and `web/.gate/status.json` | infra: `scripts/check.sh`; site: Run buttons |
| `/version.json` and the commit line in `/llms.txt`: the main commit the live site was built from | acceptance runs, anyone checking a deploy |
| `/llms.txt`, `/llms-full.txt` | the docs-only eval, outside model users |
| `/p/<id>`, `/api/share`, `/play/?code=`: share links and playground URLs | docs, the trace viewer, users |
| `web/lib/facts.json`: install commands and the version line | every page that states them |

**Site needs**

| Interface | From |
|---|---|
| `crates/cpl-wasm` (`check`, `lint`, `run`, `trace`, `lsp`, `sample`, `build`, `explain`, `stress`, `fix`) | tools, debugger, compiler |
| `cpl check`, `cpl build`, `cpl run` on the CLI | compiler |
| `cpl doc --format json`, `cpl --help`, `cpl explain --json` | tools, diagnostics |
| `spec/` markdown and `decisions.md` | spec, everyone |
| `corpus/`, bench data, `CHANGELOG.md` | idioms, perf, infra |

## Architecture

### Shape

A static site plus one Worker. `next build` (Next.js App Router, `output: 'export'`, see `web/next.config.mjs`) writes `web/out/`; the Cloudflare Worker in `web/worker/` serves it and owns the only dynamic paths, `/p/*` and `/api/*` (`web/worker/wrangler.toml`). Every compile, run, trace and stress in the browser goes through the compiler built to WebAssembly, in a Web Worker; there is no server-side compile.

`prebuild` (`web/package.json`) runs, in order: `scripts/wasm.mjs`, `wasm-size.mjs`, `releases.mjs`, `gen-ref.mjs`, `gen-tools.mjs`, `gen-debug.mjs`, `gate.mjs`. A failing gate fails the build, so nothing ships with a program the compiler rejects.

### Modules

| Path | What it is for |
|---|---|
| `web/app/(home)/page.tsx` | Landing: hero runner, syntax and library showcases, debug demo |
| `web/app/learn/`, `web/content/learn/` | Learn: read chapters (`read/`) and practice track (`practice/`) |
| `web/content/practice/<id>/` | One practice problem: starter, reference, tests; loaded by `web/lib/practice.ts` |
| `web/app/docs/`, `web/content/docs/` | Guides (`guides/`), generated reference (`ref/`), release notes |
| `web/app/play/` | Playground: `playground.tsx` wraps `Runner`, URL and share state |
| `web/app/s/` | "You found a CP Lang submission": paste generated C++, recover the source (`web/lib/extract.ts`) |
| `web/app/corpus/`, `web/lib/corpus.ts` | Problems gallery, rendered from `corpus/` |
| `web/app/performance/`, `web/lib/bench.ts` | Performance page, rendered from bench data |
| `web/app/tools/`, `web/app/setup/`, `web/lib/tools.ts` | CLI and editor pages, install |
| `web/app/llms.txt/`, `web/app/llms-full.txt/`, `web/lib/llms.ts` | Docs for models: index and one-file bundle |
| `web/app/api/search/` | Static search index (Fumadocs) |
| `web/components/runner.tsx` | The one interactive program surface: Run, Trace, Stress, C++, stdin/stdout, share |
| `web/components/live-code.tsx`, `remarkLive` in `web/source.config.ts` | Guide fences the gate measured `runs` keep their highlighted block and gain "Open in playground", plus Run when the gate's `ran` says they print something (`.gate/status.json`); WASM loads on the first Run |
| `web/components/editor/` | CodeMirror editor, CP Lang language mode, live diagnostics |
| `web/components/exercise.tsx`, `judge.tsx` | Practice problem UI and in-browser judging against tests |
| `web/components/ui/` | shadcn/ui primitives |
| `web/lib/cpl.ts` | Client API to the wasm worker (`check`, `run`, `trace`, `stress`, `build`); types imported from `crates/cpl-wasm/js/cpl.d.ts` |
| `web/lib/landing.ts` | Loads landing programs and their gate status; Run only when status is `runs` |
| `web/lib/source.ts`, `web/source.config.ts` | Fumadocs loaders for docs and Learn |
| `web/lib/layout.shared.tsx` | Site name and the one nav link list shared by every layout |
| `web/scripts/gate.mjs` | Snippet gate: every program on the site, and the browser differential |
| `web/scripts/gen-ref.mjs` | Reference pages from `spec/` and `cpl doc --format json` |
| `web/scripts/gen-tools.mjs`, `gen-debug.mjs`, `releases.mjs` | CLI metadata from `cpl --help`; trace demo; release notes from `CHANGELOG.md` |
| `web/scripts/wasm.mjs` | Copies `dist/wasm` into `web/public/wasm/` |
| `web/scripts/links.mjs` | Internal link check over `web/out` |
| `web/worker/` | Share links: `share.ts` (snapshot format, ids, limits), `index.ts` (routes, OG tags), `share.test.ts` |
| `web/tests/run.mjs`, `smoke.mjs`, `learn-smoke.mjs`, `browser.mjs` | Browser smoke: `run.mjs` serves `web/out` on a free port and runs both suites |
| `web/scripts/wasm-size.mjs` | Fails the build when `cpl.wasm` is over 1,980,000 gzipped bytes (a shrink-only cap) |
| `scripts/site-check.sh` | The site's gate: wasm build, site build (cap, snippet gate), links, browser smoke; cached |

### Adding a practice problem

1. `web/content/practice/<id>/`: `starter.cp` (what the visitor edits; it must not pass), `ref.cp` (a solution), `tests.json` (`[{ "input", "expected", "sample"? }]`, at least one `"sample": true`, shown on the page).
2. `web/content/learn/practice/<id>.mdx`: frontmatter `title` and `description: "Level A. Teaches ..."`, the statement with its sample table, then `<Judge id="<id>">` wrapping the notes shown after an accepted submission.
3. Add `<id>` to `pages` in `web/content/learn/practice/meta.json`, in level order.
4. `node web/scripts/gate.mjs` runs `ref.cp` on every test and fails when any of the three pieces is missing; `pnpm --dir web build` renders the page.

## Judgment rubric

A cold agent scores fixed samples 1-5 with a reason; Ryan's review overrides it.

1. **Does the page teach the idea**, not just list syntax? (Learn chapters, guides)
2. **Mid-contest, is the answer under 30 s away?** (search, cheat sheet, reference)
3. **Is every example honest?** It runs, its output is shown, its status is obvious.
4. **One product?** Same nav, type and spacing on every page, at phone and desktop width.
5. **Would a C++ contestant keep reading after 2 minutes?** (landing, quickstart)

Probes (cold agent, pass/fail + time):

- **P1 docs-only solve:** with only cp-lang.org, solve 3 unseen Div 2 A/B problems in the playground. Tests 1, 3.
- **P2 lookup:** find "what does `r.last_true(ok)` return on an empty range" via the site. Tests 2. Baseline: failed (search missed bare method names and core table rows); now passes, `last_true` finds the R04.4.19 row (smoke-tested).
- **P3 add a practice problem** by following ARCHITECTURE.md; the gate judges it. Files touched, minutes. Baseline (site agent dry run, not cold): ARCHITECTURE.md had no recipe and the gate checked only `ref.cp` against `tests.json`, so a problem missing its page or meta.json entry shipped silently. Now a 4-step recipe; the gate fails on a missing piece. 4 files touched. Cold run pending.
- **P4 phone pass:** list layout bugs at 390 px on every route. Tests 4.
- **P5 value in 3 minutes:** a cold reader who knows C++ opens `/` for 3 minutes, then names what CPL is, who it is for and 3 advantages over C++. Tests Milestone 1 step 1.

## Quality register

Each row is a quality a user can feel on the site, the metric, and the strongest mechanism that enforces it. Rows are grouped by the journey in `handbook/product/journeys.md` where the user feels them. `missing` means no real check exists yet. Targets and today's numbers are in `plan/domains/site/metrics.toml`. The rows were found by walking each journey and each feature the site owns, and by comparing with how mature projects measure the same thing: rustdoc doctests and docs.rs, rust-analyzer's hover and completion fixtures, TypeScript's twoslash (doc examples whose types are checked), the Rust and Go playgrounds, MDN's interactive examples, Lighthouse and Core Web Vitals budgets, and axe-core accessibility audits.

### Honesty: everything shown is true (all journeys)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Every program on the site behaves as its page claims | snippets whose measured status is below the declared one | Generated suite: every `cp` fence and `.cp` file under `web/content` is checked, built and run against its expected output | `web/scripts/gate.mjs` in prebuild, CI `site build` |
| Shown output is the real output | snippets whose printed output on the page differs from a run | [site/R1](../guidelines/site-checklist.md); Generated suite: the gate compares against the expected block | `web/scripts/gate.mjs` |
| Std reference examples run | examples shown as static code | [site/R1](../guidelines/site-checklist.md); Generated suite: every `example:` in std docs runs (print, then `assert` fallback); a compiler change reruns them | `web/scripts/gen-ref.mjs`, shrink-only list `web/scripts/doc-examples-failing.txt` |
| Doc examples type-check where they stand | std doc examples whose types or hover differ from what the text says | [site/R21](../guidelines/site-checklist.md): examples type-check and run where they stand, and hover in them comes from the language server | `web/scripts/gen-ref.mjs`, `web/tests/intelligence.mjs` |
| Lint pages show a runnable fix | lint pages whose "Write instead" cannot run | Typed random input from the lint's `read()` types, using the Stress generator | `web/scripts/lint-inputs.mjs`; **missing**: a ratchet on the count that cannot run |
| Lint pages show the lint firing | lint pages whose "Instead of" example does not raise that lint, or whose fix still raises it | [site/R2](../guidelines/site-checklist.md): the page renders the lint's own record files, which the lint suites run | `web/lib/lints.ts`, `crates/cpl-lint/src/record_tests.rs`, `crates/cpl-lint/src/checker_tests.rs` |
| Error pages show the error | error-code pages whose trigger does not raise its code, or whose fix does not compile | [site/R2](../guidelines/site-checklist.md): pages render `cpl explain --json`, whose triggers and fixes the catalogue suite runs | `web/lib/errors.ts`, `crates/cpl-cli/tests/catalogue.rs` |
| Each fact is stated once | contradicting statements of install commands, the version, retired names, std module names | [site/R2](../guidelines/site-checklist.md); Single source `web/lib/facts.json` plus a contradiction gate over every page | `web/scripts/facts.mjs`, `web/scripts/facts.test.mjs` |
| Docs agree with the compiler | names, signatures or rules in prose that the compiler rejects or that no reference page documents | Gate for std names in module tables and retired spellings; **missing**: every inline `code` that names a std item resolves | `web/scripts/facts.mjs` |
| Docs agree with `spec/` | Language chapters that state a rule differently from the spec section they cite | [site/R8](../guidelines/site-checklist.md); Process: each rule links its spec section instead of restating it; **missing**: a check that cited rules exist with the same id | `web/scripts/facts.mjs` (citations exist) |
| Citations resolve | cited spec headings or decision numbers that do not exist | Gate | `web/scripts/facts.mjs`, `web/scripts/links.mjs` |
| No broken links | broken internal link targets; external links that 404 | [site/R5](../guidelines/site-checklist.md); Generated check over every built page | `web/scripts/links.mjs` in postbuild; **missing**: external links |
| Generated pages are current | reference, lint, error, tool and release pages built from a stale `cpl` | [site/R3](../guidelines/site-checklist.md); Rebuilt every build from the `cpl` that ships; cache keyed on the binary; site scripts refuse a `cpl` older than `std/` or `crates/` | `web/scripts/cpl-bin.mjs`, `web/scripts/gen-ref.mjs`, `gen-tools.mjs`, `releases.mjs` |
| The live site is current | green main commits not live within 30 minutes | [ci/R25](../guidelines/ci-checklist.md): version.json names the built commit; the scheduled site run goes red when a commit whose site jobs passed has waited over 30 min | `.github/workflows/site.yml` step `live site is current`, `scripts/test_workflows.py` |

### J1. Discover and evaluate

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| The first screen explains CP Lang | a cold reader names what it is, who it is for, and 3 advantages over C++ after 3 minutes on `/` | Process: cold-reader probe | **missing**: not scheduled |
| The first program runs in place | landing programs without a working Run | Gate status drives Run buttons; smoke clicks Run on `/` | `web/lib/landing.ts`, `web/tests/smoke.mjs` |
| Fast first load | `/` bytes gzipped, its wasm bytes, LCP | [site/R7](../guidelines/site-checklist.md); Browser measurement; wasm loads only on first use | `web/tests/page-weight.mjs`, CI job `site browser tests` |
| Every route works on a phone | routes wider than 390 px | [site/R7](../guidelines/site-checklist.md); Browser measurement over every route | `web/tests/phone.mjs`, CI job `site browser tests` |
| Pages run without errors | console errors and failed requests per route | Browser smoke | `web/tests/smoke.mjs`, `learn-smoke.mjs`, CI job `site browser tests`; **missing** over every route |
| Links shared on social sites preview well | pages without title, description and preview image | [site/R13](../guidelines/site-checklist.md): the build fails on an exported page without its own title, description, preview image or card tag | `web/scripts/social.mjs` (site postbuild) |
| One product | layout and theme bugs at 390, 768 and 1440 px, light and dark | Process: UI pass with screenshots | **missing**: no recurring pass or screenshot diff |
| Accessible | axe-core violations per route; keyboard reach of Run, tabs and search | [site/R14](../guidelines/site-checklist.md): axe-core's WCAG 2 A and AA rules on 19 routes; today's 25 route-rule violations are listed and the list only shrinks | `web/tests/a11y.mjs`, CI job `site browser tests` |

### J2. Learn

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Productive in under an hour | intro sections, examples that run, time for a cold reader to solve the first practice problem | Gate for the examples; process for the time | `web/scripts/gate.mjs`; **missing**: timed cold run |
| Learn covers the ground | judged problems per level, Div 2 A to Div 1 C; chapters per topic | Count | `plan/domains/site/metrics.toml` |
| Practice problems are judged correctly | reference solutions failing a test; starters that pass; problems missing a page, meta entry or sample | Generated suite over every problem | `web/scripts/gate.mjs` |
| Practice tests are strong | wrong solutions the tests accept | [site/R1](../guidelines/site-checklist.md): every one-token mutant of each `ref.cp` must fail a test, unless listed with the reason no test can | `web/scripts/practice-mutants.mjs` (prebuild) |
| Hints teach before acceptance | problems without a hint linking std docs or a guide | [site/R19](../guidelines/site-checklist.md): the gate fails on a problem whose hints link nowhere | `web/scripts/gate.mjs` (site prebuild) |
| Chapters teach the design | chapters without runnable examples and a "why, compared with X" section | Process: [Judgment rubric](#judgment-rubric) | partial: the gate checks the examples |
| Design chapters cover the whole language | `spec/core` parts no Language chapter cites | [site/R8](../guidelines/site-checklist.md); Gate | `web/scripts/facts.mjs` |
| Every std item and lint is documented | std items with no reference entry; entries without cost, empty, order and traps | Generated from `cpl doc --format json` | `web/scripts/gen-ref.mjs`; **missing**: fail on an entry without its sections |
| Std reference is readable | entries without a highlighted signature, parameter table and labelled cost, empty, order and traps rows | [site/R4](../guidelines/site-checklist.md): the reference build fails on an entry with no signature, a fn whose parameters don't parse into a table, a `cost:`/`empty:`/`order:`/`ties:`/`traps:` line left in the prose, or a cost that KaTeX can't render | `web/scripts/gen-ref.mjs` (site prebuild) |
| Costs read as math | cost fields shown as raw text | KaTeX rendering of cost fields | `web/scripts/gen-ref.mjs`; **missing**: a check that every cost parses |
| Search finds what a contestant types | std items, lints, error codes and guide headings not found by their name | Smoke on sample queries | `web/tests/smoke.mjs`; **missing**: every name searched |
| Search finds the abstraction without its name | task phrasings ("binary search on answer", "prefix sums") whose first results miss the right item | [site/R18](../guidelines/site-checklist.md): task phrasings with the pages that answer them, through the real search dialog | `web/tests/search-tasks.mjs` |
| Search stays small | search index bytes gzipped | Budget | `web/scripts/budgets.mjs` in postbuild |
| Writable from the docs | docs-only eval pass rate | Process: a model solves unseen problems from `/llms-full.txt` | `scripts/writable-from-docs.py`; **missing**: it cannot run (no statements, no model key) |
| The docs bundle for models is complete | std items, guides and chapters missing from `/llms-full.txt` | [site/R2](../guidelines/site-checklist.md): the build fails when a docs or Learn page it exports is not in the one-file bundle; the five pages drawn from code are on a list that only shrinks | `web/scripts/llms-complete.mjs` (site postbuild) |

### J3. Install and configure

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| One correct install command per platform and channel | install commands that differ from `web/lib/facts.json` | Single source plus contradiction gate | `web/scripts/facts.mjs` |
| The installers are served | `/install.sh` and `/install.ps1` missing or empty | CI step | CI `site build`, deploy smoke in `.github/workflows/site.yml` |
| Editor setup steps work | editor tabs whose steps fail on a clean machine | Infra's acceptance run per platform | `research/acceptance/ci-steps.sh` (infra) |

### J5. Write in the browser editor

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Live diagnostics while typing | edits whose errors appear after more than 500 ms; errors that differ from `cpl check` | [site/R6](../guidelines/site-checklist.md): for a sample of the error registry's triggers, the editor shows the code `cpl check` raises within 1.5 s of the last keystroke | `web/tests/diagnostics.mjs`, CI job `site browser tests` |
| Browser editor matches the language server | LSP features (hover, completion, definition, inlay hints, signature help, rename, formatting) the browser editor lacks | [site/R6](../guidelines/site-checklist.md); the server's advertised capabilities, read from the browser build, are each sent by the editor (`browser_lsp_gaps`, hard 0); tool data in `web/` is never re-derived (`site_rederived`, hard 0); **missing**: per-request answer parity on fixtures | `web/lib/lsp-parity.test.ts`, `web/tests/intelligence.mjs` |
| Hover and completion work on every std item | std items with no full-signature hover, or no completion after `.` (including after a call) | [site/R6](../guidelines/site-checklist.md); tools generates it from the `cpl doc` extraction and `crates/cpl-wasm/tests/parity.rs` holds the browser to `cpl lsp`; **missing** until the suite lands | `crates/cpl-lsp/tests/std_items.rs` (tools) |
| Completion works mid-edit | positions in a corpus of half-written programs where completion returns nothing | [site/R6](../guidelines/site-checklist.md): every hill-climb port retyped one identifier at a time, completion must offer the identifier typed next; recall per kind only goes up, and the browser answers as `cpl lsp` does | `crates/cpl-lsp/tests/completion_replay.rs` (tools), `crates/cpl-wasm/tests/parity.rs` |
| From code to docs in one click | std names in the editor that do not open their reference entry; Cmd+click that does not open a new tab | [site/R5](../guidelines/site-checklist.md); Cmd+click in an editor follows the language server's definition; search results open in a new tab on Cmd+click or middle click | `web/tests/intelligence.mjs`; **missing**: every std name |
| Inlay type hints | untyped bindings without a hint; hints that cannot be switched off | [site/R6](../guidelines/site-checklist.md); the browser test sees a hint on an untyped binding, switches hints off, reloads and sees none | `web/tests/intelligence.mjs` |
| Typing feels like a Python editor | indentation cases | Hand-written unit suite | `web/components/editor/indent.test.ts` |
| Highlighting matches the language | tokens the editor colours differently from the TextMate grammar | [site/R16](../guidelines/site-checklist.md): differential over `std/` and the ports, word by word | `web/components/editor/highlight-parity.test.ts` |
| Lints and fixes in the editor | lints `cpl lint` raises that the editor does not show; quick fixes that do not apply | [site/R6](../guidelines/site-checklist.md): for a sample of the lint registry's triggers, the editor shows the code `cpl lint` raises, and clicking its fix gives the text `cpl lint`'s edits give | `web/tests/lint-fixes.mjs`, CI job `site browser tests` |
| Work is not lost | edits lost on reload or navigation | [site/R12](../guidelines/site-checklist.md): every editor keeps its draft per page and starting text | `web/lib/draft.ts`, `web/tests/drafts.mjs`, CI job `site browser tests` |

### J6 and J7. Run, test, stress and trace in the browser

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Run gives the CLI's answer | programs whose browser output, trap or exit differs from `cpl run` | [site/R1](../guidelines/site-checklist.md); every program the gate runs also runs in `cpl-build.wasm`, and a different output or exit fails the build; programs too slow for the browser's time limit are listed in `web/.gate/browser-slow.json` | `web/scripts/gate.mjs`, `crates/cpl-wasm/tests/parity.rs` |
| Runaway programs stop cleanly | infinite loops or huge output that freeze the page; time to a usable editor after a timeout | Runs in a Web Worker with a 10 s limit | `web/lib/cpl.ts`; **missing**: a browser test |
| Traps read in source terms | traps shown without the line and local values | Shown from the wasm trap | `web/tests/smoke.mjs` (one trap) |
| Stress finds the counterexample | planted bugs Stress misses in the browser | [site/R15](../guidelines/site-checklist.md): the browser's stress and `cpl stress` agree on verdict, shrunk input and first diverging variable for every `@alt` program, and a browser run over 20 s for 30 cases fails; time-limit counterexamples are skipped until the browser shrinks them quickly | `web/scripts/stress-parity.mjs` (site prebuild), `web/tests/smoke.mjs` |
| Trace opens | Trace runs that fail to open the viewer | Browser smoke | `web/tests/smoke.mjs` |
| Build and Explain load on demand | first Build, C++ or Explain that fails to load `cpl-build.wasm` | Browser test | `web/tests/smoke.mjs` |
| Share links stay valid | snapshot format, id and size limit regressions; old links that stop opening | Hand-written unit suite; smoke round trip | `web/worker/share.test.ts`, `web/tests/smoke.mjs` |
| The site keeps working when a service fails | pages that break when the share API or wasm fetch fails | [site/R17](../guidelines/site-checklist.md): browser test with each service made to fail | `web/tests/resilience.mjs` |

### J9. Submit

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Browser C++ matches the CLI | programs whose browser C++ differs from `cpl build` | [site/R1](../guidelines/site-checklist.md); every program the gate builds is built again by `cpl-build.wasm`, and any byte of difference fails the build | `web/scripts/gate.mjs`, `crates/cpl-wasm/tests/parity.rs` |
| Submissions can be traced back | generated C++ whose source cannot be recovered on `/s` | Hand-written unit suite | `web/lib/extract.test.ts` |

### Contributors

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Editors load fast | `cpl.wasm` bytes gzipped | [site/R7](../guidelines/site-checklist.md); Re-architecture: codegen, Explain and Stress live in the lazily loaded `cpl-build.wasm`; shrink-only cap that prints the top contributors when it fails | `web/scripts/wasm-size.mjs` in prebuild |
| Client types match the wasm API | a copied types file | No copy: `web/lib/cpl.ts` imports `crates/cpl-wasm/js/cpl.d.ts` | `web/lib/cpl.ts` |
| The site check is fast | `site build` CI job seconds; per-stage seconds | Stage timings and a named kill for a stage that hangs | `web/scripts/prebuild.mjs`; **missing**: a budget on the job time |
| Site code is tested | unit coverage of `web/lib`, `web/components`, `web/scripts` | [site/R20](../guidelines/site-checklist.md): unit tests measure coverage and fail below a floor that only rises; 7.8% of statements today | `web/vitest.config.mts` (`pnpm --dir web test`) |
| Adding a page or problem is mechanical | steps to add a practice problem; pieces the gate does not check | Recipe plus gate | [Adding a practice problem](#adding-a-practice-problem), `web/scripts/gate.mjs` |

## Suites and tools

| Command | What it catches |
|---|---|
| `pnpm --dir web build` | Runs prebuild (stage chains in `web/scripts/prebuild.mjs`: wasm copy, size cap, lint inputs, reference, tools, trace demo, facts gate, snippet gate), then `next build`, then the link check and search budget |
| `node web/scripts/gate.mjs` | Programs that don't behave as declared; incomplete practice problems. Writes `web/.gate/status.json` and `report.md` |
| `node web/scripts/facts.mjs` | Contradicting facts, retired names, uncited spec parts |
| `node web/scripts/wasm-size.mjs [--why]` | `cpl.wasm` over its cap; `--why` lists bytes per crate |
| `pnpm --dir web test` | Unit suites: indentation, share links, C++ extraction, facts rules |
| `pnpm --dir web typecheck` | Type errors in the site and generated types |
| `pnpm --dir web smoke` | Page errors, Run, Trace, search samples, share links, editor intelligence, in Chromium |
| `pnpm --dir web phone` | Routes that overflow 390 px |
| `pnpm --dir web page-weight` | Bytes and wasm loaded per page |
| `scripts/site-check.sh` | All of the above in order, cached |

Set `CPL=<path to cpl>` to use a specific compiler build. `PREBUILD_STAGE_SECONDS` sets the per-stage limit, 240 s by default.

## Changes

- 2026-11-23: first charter, folded from the site plan's README and ARCHITECTURE.
- 2026-11-23: register made exhaustive, grouped by journey, with prior art.
- 2026-11-23: architecture, the practice-problem recipe and the judgment rubric folded in from `plan/domains/site/`.
- 2026-11-23: prose overview first; rules moved to `handbook/guidelines/site-checklist.md`.

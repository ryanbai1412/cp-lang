# Domain: site

| | |
|---|---|
| Kind | Domain charter |
| Owner | site |
| Status | Active |
| Describes | `web/`, `scripts/site-check.sh` |

## Mission

A competitive programmer goes from "what is this" to an accepted submission they trust using cp-lang.org alone. The site teaches CP Lang from a first program to the whole design. Every program it shows is checked by the compiler in the build that ships it, and every tool (check, lint, run, trace, stress, explain, build) works in the browser. The bar: **a cold reader, person or model, writes correct CP Lang from the docs alone.**

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

## Quality register

Each row is a quality a user can feel on the site, the metric, and the strongest mechanism that enforces it. Rows are grouped by the journey in `handbook/product/journeys.md` where the user feels them. `missing` means no real check exists yet. Targets and today's numbers are in `plan/domains/site/metrics.toml`. The rows were found by walking each journey and each feature the site owns, and by comparing with how mature projects measure the same thing: rustdoc doctests and docs.rs, rust-analyzer's hover and completion fixtures, TypeScript's twoslash (doc examples whose types are checked), the Rust and Go playgrounds, MDN's interactive examples, Lighthouse and Core Web Vitals budgets, and axe-core accessibility audits.

### Honesty: everything shown is true (all journeys)

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Every program on the site behaves as its page claims | snippets whose measured status is below the declared one | Generated suite: every `cp` fence and `.cp` file under `web/content` is checked, built and run against its expected output | `web/scripts/gate.mjs` in prebuild, CI `site build` |
| Shown output is the real output | snippets whose printed output on the page differs from a run | Generated suite: the gate compares against the expected block | `web/scripts/gate.mjs` |
| Std reference examples run | examples shown as static code | Generated suite: every `example:` in std docs runs (print, then `assert` fallback); a compiler change reruns them | `web/scripts/gen-ref.mjs`; **missing**: a shrink-only ratchet on the static count |
| Doc examples type-check where they stand | std doc examples whose types or hover differ from what the text says | **missing**: twoslash-style checked annotations | none |
| Lint pages show a runnable fix | lint pages whose "Write instead" cannot run | Typed random input from the lint's `read()` types, using the Stress generator | `web/scripts/lint-inputs.mjs`; **missing**: a ratchet on the count that cannot run |
| Lint pages show the lint firing | lint pages whose "Instead of" example does not raise that lint, or whose fix still raises it | **missing**: run `cpl lint` on both examples in the build | none |
| Error pages show the error | error-code pages whose trigger does not raise its code, or whose fix does not compile | Registry tests (diagnostics); **missing** on the rendered page | `crates/cpl-diag` tests |
| Each fact is stated once | contradicting statements of install commands, the version, retired names, std module names | Single source `web/lib/facts.json` plus a contradiction gate over every page | `web/scripts/facts.mjs`, `web/scripts/facts.test.mjs` |
| Docs agree with the compiler | names, signatures or rules in prose that the compiler rejects or that no reference page documents | Gate for std names in module tables and retired spellings; **missing**: every inline `code` that names a std item resolves | `web/scripts/facts.mjs` |
| Docs agree with `spec/` | Language chapters that state a rule differently from the spec section they cite | Process: each rule links its spec section instead of restating it; **missing**: a check that cited rules exist with the same id | `web/scripts/facts.mjs` (citations exist) |
| Citations resolve | cited spec headings or decision numbers that do not exist | Gate | `web/scripts/facts.mjs`, `web/scripts/links.mjs` |
| No broken links | broken internal link targets; external links that 404 | Generated check over every built page | `web/scripts/links.mjs` in postbuild; **missing**: external links |
| Generated pages are current | reference, lint, error, tool and release pages built from a stale `cpl` | Rebuilt every build from the `cpl` that ships; cache keyed on the binary | `web/scripts/gen-ref.mjs`, `gen-tools.mjs`, `releases.mjs` |
| The live site is current | green main commits not live within 30 minutes | `/version.json` names the built commit | **missing**: an alert when it falls behind |

### J1. Discover and evaluate

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| The first screen explains CP Lang | a cold reader names what it is, who it is for, and 3 advantages over C++ after 3 minutes on `/` | Process: cold-reader probe | **missing**: not scheduled |
| The first program runs in place | landing programs without a working Run | Gate status drives Run buttons; smoke clicks Run on `/` | `web/lib/landing.ts`, `web/tests/smoke.mjs` |
| Fast first load | `/` bytes gzipped, its wasm bytes, LCP | Browser measurement; wasm loads only on first use | `web/tests/page-weight.mjs`; **missing** in CI |
| Every route works on a phone | routes wider than 390 px | Browser measurement over every route | `web/tests/phone.mjs`; **missing** in CI |
| Pages run without errors | console errors and failed requests per route | Browser smoke | `web/tests/smoke.mjs`, `learn-smoke.mjs`; **missing** in CI and over every route |
| Links shared on social sites preview well | pages without title, description and preview image | **missing** | none |
| One product | layout and theme bugs at 390, 768 and 1440 px, light and dark | Process: UI pass with screenshots | **missing**: no recurring pass or screenshot diff |
| Accessible | axe-core violations per route; keyboard reach of Run, tabs and search | **missing** | none |

### J2. Learn

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Productive in under an hour | intro sections, examples that run, time for a cold reader to solve the first practice problem | Gate for the examples; process for the time | `web/scripts/gate.mjs`; **missing**: timed cold run |
| Learn covers the ground | judged problems per level, Div 2 A to Div 1 C; chapters per topic | Count | `plan/domains/site/metrics.toml` |
| Practice problems are judged correctly | reference solutions failing a test; starters that pass; problems missing a page, meta entry or sample | Generated suite over every problem | `web/scripts/gate.mjs` |
| Practice tests are strong | wrong solutions the tests accept | **missing**: mutate each reference solution and require a test to fail | none |
| Hints teach before acceptance | problems without a hint linking std docs or a guide | Gate | **missing**: hints are written, not checked |
| Chapters teach the design | chapters without runnable examples and a "why, compared with X" section | Process: judgment rubric in `plan/domains/site/README.md` | partial: the gate checks the examples |
| Design chapters cover the whole language | `spec/core` parts no Language chapter cites | Gate | `web/scripts/facts.mjs` |
| Every std item and lint is documented | std items with no reference entry; entries without cost, empty, order and traps | Generated from `cpl doc --format json` | `web/scripts/gen-ref.mjs`; **missing**: fail on an entry without its sections |
| Std reference is readable | entries without a highlighted signature, parameter table and labelled cost, empty, order and traps rows | **missing**: render from structured `cpl doc` fields, and fail the build on an unstructured entry | none |
| Costs read as math | cost fields shown as raw text | KaTeX rendering of cost fields | `web/scripts/gen-ref.mjs`; **missing**: a check that every cost parses |
| Search finds what a contestant types | std items, lints, error codes and guide headings not found by their name | Smoke on sample queries | `web/tests/smoke.mjs`; **missing**: every name searched |
| Search finds the abstraction without its name | task phrasings ("binary search on answer", "prefix sums") whose first results miss the right item | **missing**: a query set with expected results | none |
| Search stays small | search index bytes gzipped | Budget | `web/scripts/budgets.mjs` in postbuild |
| Writable from the docs | docs-only eval pass rate | Process: a model solves unseen problems from `/llms-full.txt` | `scripts/writable-from-docs.py`; **missing**: it cannot run (no statements, no model key) |
| The docs bundle for models is complete | std items, guides and chapters missing from `/llms-full.txt` | **missing** | none |

### J3. Install and configure

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| One correct install command per platform and channel | install commands that differ from `web/lib/facts.json` | Single source plus contradiction gate | `web/scripts/facts.mjs` |
| The installers are served | `/install.sh` and `/install.ps1` missing or empty | CI step | CI `site build`, deploy smoke in `.github/workflows/site.yml` |
| Editor setup steps work | editor tabs whose steps fail on a clean machine | Infra's acceptance run per platform | `research/acceptance/ci-steps.sh` (infra) |

### J5. Write in the browser editor

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Live diagnostics while typing | edits whose errors appear after more than 500 ms; errors that differ from `cpl check` | Same analysis as the CLI, in wasm | **missing**: a latency measurement and a differential against `cpl check` |
| Browser editor matches the language server | LSP features (hover, completion, definition, inlay hints, signature help, rename, formatting) the browser editor lacks | **missing**: a parity check generated from the server's capabilities | `web/tests/intelligence.mjs` covers one hover, completion and definition |
| Hover and completion work on every std item | std items with no full-signature hover, or no completion after `.` (including after a call) | **missing**: a suite generated from `cpl doc --format json` (shared with tools) | none |
| Completion works mid-edit | positions in a corpus of half-written programs where completion returns nothing | **missing**: a fixture suite like rust-analyzer's | none |
| From code to docs in one click | std names in the editor that do not open their reference entry; Cmd+click that does not open a new tab | **missing** | none |
| Inlay type hints | untyped bindings without a hint; hints that cannot be switched off | **missing** | none |
| Typing feels like a Python editor | indentation cases | Hand-written unit suite | `web/components/editor/indent.test.ts` |
| Highlighting matches the language | tokens the editor colours differently from the TextMate grammar | **missing**: a differential over the gate's snippets | none |
| Lints and fixes in the editor | lints `cpl lint` raises that the editor does not show; quick fixes that do not apply | **missing** | none |
| Work is not lost | edits lost on reload or navigation | Draft kept in the URL or local storage | **missing**: a browser test |

### J6 and J7. Run, test, stress and trace in the browser

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Run gives the CLI's answer | programs whose browser output, trap or exit differs from `cpl run` | **missing**: a differential over the gate's snippets in wasm | none |
| Runaway programs stop cleanly | infinite loops or huge output that freeze the page; time to a usable editor after a timeout | Runs in a Web Worker with a 10 s limit | `web/lib/cpl.ts`; **missing**: a browser test |
| Traps read in source terms | traps shown without the line and local values | Shown from the wasm trap | `web/tests/smoke.mjs` (one trap) |
| Stress finds the counterexample | planted bugs Stress misses in the browser | Debugger's suite for the engine; **missing**: browser test of the Stress tab | none |
| Trace opens | Trace runs that fail to open the viewer | Browser smoke | `web/tests/smoke.mjs` |
| Build and Explain load on demand | first Build, C++ or Explain that fails to load `cpl-build.wasm` | Browser test | `web/tests/smoke.mjs` |
| Share links stay valid | snapshot format, id and size limit regressions; old links that stop opening | Hand-written unit suite; smoke round trip | `web/worker/share.test.ts`, `web/tests/smoke.mjs` |
| The site keeps working when a service fails | pages that break when the share API or wasm fetch fails | **missing** | none |

### J9. Submit

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Browser C++ matches the CLI | programs whose browser C++ differs from `cpl build` | **missing**: a differential over the gate's snippets | none |
| Submissions can be traced back | generated C++ whose source cannot be recovered on `/s` | Hand-written unit suite | `web/lib/extract.test.ts` |

### Contributors

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Editors load fast | `cpl.wasm` bytes gzipped | Re-architecture: codegen, Explain and Stress live in the lazily loaded `cpl-build.wasm`; shrink-only cap that prints the top contributors when it fails | `web/scripts/wasm-size.mjs` in prebuild |
| Client types match the wasm API | stale `web/lib/cpl-types.ts` | Generated from the wasm build each build | `web/scripts/wasm.mjs` |
| The site check is fast | `site build` CI job seconds; per-stage seconds | Stage timings and a named kill for a stage that hangs | `web/scripts/prebuild.mjs`; **missing**: a budget on the job time |
| Site code is tested | unit coverage of `web/lib`, `web/components`, `web/scripts` | Hand-written unit suites | `pnpm --dir web test`; **missing**: coverage measurement |
| Adding a page or problem is mechanical | steps to add a practice problem; pieces the gate does not check | Recipe plus gate | `plan/domains/site/ARCHITECTURE.md`, `web/scripts/gate.mjs` |

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

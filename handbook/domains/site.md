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

Each row is a quality the site must be high on, the metric, and the strongest mechanism that enforces it. `missing` means no real check exists yet. Targets and today's numbers are in `plan/domains/site/metrics.toml`.

| Quality | Metric | Mechanism | Check |
|---|---|---|---|
| Every program on the site behaves as its page claims | snippets whose measured status is below the declared one | Generated suite: every `cp` fence and `.cp` file under `web/content` is checked, built and run against its expected output | `web/scripts/gate.mjs` in prebuild, CI `site build` |
| Practice problems are judged correctly | reference solutions failing a test; starters that pass | Generated suite: each `ref.cp` runs on every case of its `tests.json`; a problem missing its page, meta entry or sample fails | `web/scripts/gate.mjs` |
| Std reference examples run | examples shown as static code | Generated suite: every `example:` in std docs runs (print, then `assert` fallback); a compiler change reruns them | `web/scripts/gen-ref.mjs`; **missing**: a shrink-only ratchet on the static count |
| Lint pages show a runnable fix | lint pages whose "Write instead" cannot run | Typed random input from the lint's `read()` types, using the Stress generator | `web/scripts/lint-inputs.mjs`; **missing**: a ratchet on the count that cannot run |
| Each fact is stated once | contradicting statements of install commands, the version, retired names, std module names | Single source `web/lib/facts.json` plus a contradiction gate over every page | `web/scripts/facts.mjs`, `web/scripts/facts.test.mjs` |
| Design chapters cover the whole language | `spec/core` parts no Language chapter cites | Gate | `web/scripts/facts.mjs` |
| Citations resolve | cited spec headings or decision numbers that do not exist | Gate | `web/scripts/facts.mjs`, `web/scripts/links.mjs` |
| No broken links | broken internal link targets | Generated check over every built page | `web/scripts/links.mjs` in postbuild |
| Search finds what a contestant types | std items not found by their name | Smoke on sample queries | `web/tests/smoke.mjs`; **missing**: every std item and lint searched by name |
| Search stays small | search index bytes gzipped | Budget | `web/scripts/budgets.mjs` in postbuild |
| Editors load fast | `cpl.wasm` bytes gzipped | Re-architecture: codegen, Explain and Stress live in the lazily loaded `cpl-build.wasm`; shrink-only cap that prints the top contributors when it fails | `web/scripts/wasm-size.mjs` in prebuild |
| Pages that only show code load no wasm | `/` bytes gzipped and its wasm bytes | Browser measurement | `web/tests/page-weight.mjs`; **missing** in CI |
| Every route works on a phone | routes wider than 390 px | Browser measurement over every route | `web/tests/phone.mjs`; **missing** in CI |
| Pages run without errors | page errors; Run, Trace, share round trip | Browser smoke | `web/tests/smoke.mjs`, `learn-smoke.mjs`; **missing** in CI |
| Browser editor matches the language server | LSP features (hover, completion, definition, inlay hints, signature help) the browser editor lacks | **missing**: a parity check generated from the server's capabilities | `web/tests/intelligence.mjs` covers one hover, completion and definition |
| Hover and completion work on every std item | std items with no full-signature hover, or no completion after `.` (including after a call) | **missing**: a suite generated from `cpl doc --format json` (shared with tools) | none |
| From code to docs in one click | std names in the editor that do not open their reference entry | **missing** | none |
| Std reference is readable | entries without a highlighted signature, parameter table and labelled cost, empty, order and traps rows | **missing**: render from structured `cpl doc` fields, and fail the build on an unstructured entry | none |
| Editor typing feels like a Python editor | indentation cases | Hand-written unit suite | `web/components/editor/indent.test.ts` |
| Share links stay valid | snapshot format, id and size limit regressions | Hand-written unit suite | `web/worker/share.test.ts` |
| Submissions can be traced back | generated C++ whose source cannot be recovered | Hand-written unit suite | `web/lib/extract.test.ts` |
| Client types match the wasm API | stale `web/lib/cpl-types.ts` | Generated from the wasm build each build | `web/scripts/wasm.mjs` |
| The live site is current | green main commits not live within 30 minutes | `/version.json` names the built commit | **missing**: an alert when it falls behind |
| The site check is fast | `site build` CI job minutes; per-stage seconds | Stage timings and a named kill for a stage that hangs | `web/scripts/prebuild.mjs`; **missing**: a budget on the job time |
| Writable from the docs | docs-only eval pass rate | Process: a model solves unseen problems from `/llms-full.txt` | `scripts/writable-from-docs.py`; **missing**: it cannot run (no statements, no model key) |
| A newcomer sees the value fast | a cold reader names what CP Lang is and 3 advantages over C++ after 3 minutes on `/` | Process: cold-reader probe | **missing**: not scheduled |
| Chapters teach the design | chapters without examples that run and a "why, compared with X" section | Process: judgment rubric in `plan/domains/site/README.md` | partial: the gate checks the examples |
| One product | layout and theme bugs at 390, 768 and 1440 px, light and dark | Process: UI pass with screenshots | **missing**: no recurring pass |
| Accessible | contrast, keyboard reach, labels | **missing** | none |

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

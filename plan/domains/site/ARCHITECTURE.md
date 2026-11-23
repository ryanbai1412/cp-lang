# site architecture

The code in `web/` as it stands (read 2026-11-22, main `4b8f504f`). The site lane keeps this current in the same commit as any structural change.

## Shape

A static site plus one Worker. `next build` (Next.js App Router, `output: 'export'`, see `web/next.config.mjs`) writes `web/out/`; the Cloudflare Worker in `web/worker/` serves it and owns the only dynamic paths, `/p/*` and `/api/*` (`web/worker/wrangler.toml`). Every compile, run, trace and stress in the browser goes through the compiler built to WebAssembly, in a Web Worker; there is no server-side compile.

`prebuild` (`web/package.json`) runs, in order: `scripts/wasm.mjs`, `wasm-size.mjs`, `releases.mjs`, `gen-ref.mjs`, `gen-tools.mjs`, `gen-debug.mjs`, `gate.mjs`. A failing gate fails the build, so nothing ships with a program the compiler rejects.

## Modules

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
| `web/lib/cpl.ts`, `web/lib/cpl-types.ts` | Client API to the wasm worker (`check`, `run`, `trace`, `stress`, `build`); types generated |
| `web/lib/landing.ts` | Loads landing programs and their gate status; Run only when status is `runs` |
| `web/lib/source.ts`, `web/source.config.ts` | Fumadocs loaders for docs and Learn |
| `web/lib/layout.shared.tsx` | Site name and the one nav link list shared by every layout |
| `web/scripts/gate.mjs` | Snippet gate (below) |
| `web/scripts/gen-ref.mjs` | Reference pages from `spec/` and `cpl doc --format json` |
| `web/scripts/gen-tools.mjs`, `gen-debug.mjs`, `releases.mjs` | CLI metadata from `cpl --help`; trace demo; release notes from `CHANGELOG.md` |
| `web/scripts/wasm.mjs` | Copies `dist/wasm` into `web/public/wasm/`, regenerates `cpl-types.ts` |
| `web/scripts/links.mjs` | Internal link check over `web/out` |
| `web/worker/` | Share links: `share.ts` (snapshot format, ids, limits), `index.ts` (routes, OG tags), `share.test.ts` |
| `web/tests/run.mjs`, `smoke.mjs`, `learn-smoke.mjs`, `browser.mjs` | Browser smoke: `run.mjs` serves `web/out` on a free port and runs both suites |
| `web/scripts/wasm-size.mjs` | Fails the build when `cpl.wasm` is over 1,980,000 gzipped bytes (a shrink-only cap) |
| `scripts/site-check.sh` | The site's gate: wasm build, site build (cap, snippet gate), links, browser smoke; cached |

## The snippet gate

`web/scripts/gate.mjs` finds every `.cp` file and every fenced `cp` block in `web/content`, then runs `cpl check`, `cpl build`, and `cpl run` against the expected output when there is one. Each snippet declares a status: `runs` (runs and matches), `checks` (type-checks, not runnable yet), `spec` (0.3 syntax the compiler does not accept yet). Measured status below declared fails. It writes `web/.gate/status.json` (read by `web/lib/landing.ts`), `report.md`, and `corpus.json`. It runs in `prebuild` and in `scripts/check.sh`.

## Adding a practice problem

1. `web/content/practice/<id>/`: `starter.cp` (what the visitor edits; it must not pass), `ref.cp` (a solution), `tests.json` (`[{ "input", "expected", "sample"? }]`, at least one `"sample": true`, shown on the page).
2. `web/content/learn/practice/<id>.mdx`: frontmatter `title` and `description: "Level A. Teaches ..."`, the statement with its sample table, then `<Judge id="<id>">` wrapping the notes shown after an accepted submission.
3. Add `<id>` to `pages` in `web/content/learn/practice/meta.json`, in level order.
4. `node web/scripts/gate.mjs` runs `ref.cp` on every test and fails when any of the three pieces is missing; `pnpm --dir web build` renders the page.

## Interfaces other domains use

| Interface | Consumers |
|---|---|
| `/llms.txt`, `/llms-full.txt` (docs bundle) | idioms / infra: docs-only eval; outside model users |
| `web/out/` static export + `web/worker/wrangler.toml` | infra: deploy (`.github/workflows/site.yml`) |
| `node web/scripts/gate.mjs` exit code | infra: `scripts/check.sh` |
| `/p/<id>`, `/api/share`, `/api/p/<id>` | anyone sharing a program; `cpl` could post to it later |
| `/play/?code=` URL state | links from docs, guides, trace viewer |

## Interfaces the site consumes

| Interface | Owner | Used by |
|---|---|---|
| `crates/cpl-wasm` via `scripts/build-wasm.sh` (`spawn`, `check`, `run`, `trace`, `stress`, `build`) | compiler / tools / debugger | `web/lib/cpl.ts` |
| `cpl check/build/run` CLI | compiler | `web/scripts/gate.mjs` |
| `cpl doc --format json`, `spec/` markdown | tools, spec | `web/scripts/gen-ref.mjs` |
| `cpl --help` | tools | `web/scripts/gen-tools.mjs` |
| `corpus/`, bench data, `CHANGELOG.md` | idioms, perf, infra | gallery, performance, release notes |

## Invariants

- Every program shown on the site passed the gate in the build that shipped it; a Run button appears only for status `runs`.
- No server compile: user code runs only in the browser's wasm worker (10 s timeout, `web/lib/cpl.ts`).
- Showing code costs no wasm: an editor checks only after an edit, a Run, or with `checkOnLoad` (the playground and `/s`), so `/` and Learn fetch `cpl.wasm` on first use (`web/components/editor/code-editor.tsx`; the browser smoke asserts it for `/`).
- Generated content (`content/docs/ref`, `content/docs/releases.mdx`, `content/generated/`) is rebuilt every build; edit the source, not the output.
- Share snapshots are immutable, versioned, at most 64 KB (`web/worker/share.ts`).
- One nav list for every layout (`web/lib/layout.shared.tsx`).

## Known debt

- Search index 3.0 MB raw, 487 KB gzipped at level 9 (`web/out/api/search`, from `web/app/api/search/route.ts`): reference pages index their headings and intro only (core chapters also their table rows, without the repeated header), so prose inside a reference entry is not searchable; the tokenizer also indexes each part of a dotted name, so `last_true` finds `SortedBag.last_true`. Adds `zbsearch` (the version fumadocs-core uses) as a direct dependency for that tokenizer. The `search` step of `web/tests/smoke.mjs` guards a reference item, a guide heading and a core row. 13 KB under the 500 KB target: the next cut is per-entry term frequencies (140 KB gzipped of the index).
- Two layouts, one header order: `HomeLayout` on Home, Play, Performance, Tools and the Learn index; notebook `DocsLayout` on Docs and Learn chapters, whose header `notebookOptions()` (`web/lib/layout.shared.tsx`) and a `#nd-subnav` rule in `web/app/global.css` rearrange to match (title and links, then search and icons). The rule depends on Fumadocs' header markup; on a Fumadocs upgrade, check the docs header against the landing's. Phones still show two menu icons (chevron on Home layouts, sidebar on Docs).
- Browser smoke and `scripts/site-check.sh` are not in CI (`.github/workflows/site.yml` curls 4 routes only).
- `web/scripts/import-guides.py` is a one-off migration that reads a `site/` directory which no longer exists.
- The docs-only eval cannot run: `.github/workflows/writable-from-docs.yml` calls `scripts/fetch-statement.py`, deleted in `e4fc2a14`, and `corpus/statements/` is absent.
- Only 4 unit tests (`web/lib/extract.test.ts`, `web/worker/share.test.ts`); `runner.tsx` and the gate have none.

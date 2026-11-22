# Build, checks, hosting, phases

Status: proposed.

## Layout

```
web/
  app/                     # Next.js App Router routes
    (marketing)/page.tsx   # /
    learn/ docs/ play/ p/ s/ setup/ tools/ problems/ ...
  components/ui/           # shadcn, as generated
  components/              # editor, verdict, compare, api-entry
  content/                 # hand-written MDX, landing snippets,
    problems/<id>/         # course problems
  loaders/                 # build-time loaders (TypeScript)
  lib/wasm/                # worker + bindings for crates/cpl-wasm
  worker/                  # Cloudflare Worker for /p/ links
  tests/                   # vitest + playwright
  package.json  next.config.ts  tsconfig.json
```

`output: "export"` in `next.config.ts`; `pnpm build` writes `web/out/`.

## Loaders (build time, no regex over Rust)

```
stdlib    <- CPL_STD=spec/std cpl doc --format json stdlib
errors    <- cpl explain --json
cli       <- cpl <cmd> --help
judges    <- cpl judges
spec      <- spec/core/*.md, spec/lib/*.md, spec/tools/*.md
glossary  <- spec/GLOSSARY.md
decisions <- decisions.md
domains   <- plan/DOMAINS.md
corpus    <- corpus/index.json + corpus/problems/*/{meta.json,spec.cp}
perf      <- bench/corpus-p95-results.json
editors   <- editors/*/README.md
```

Each loader has a unit test against a fixture and fails the build on a
missing or malformed source.

## The snippet gate

`web/scripts/gate.ts`, run by `pnpm gate` and from
`scripts/check.sh --fast`:

1. Collect every CP Lang fence from MDX, landing snippets, problems and
   generated pages, with its expected output if given.
2. For each: `cpl check`; if it passes, `cpl run` on its input and
   compare output, then `cpl build` + g++. Pass all -> `runs`.
3. Otherwise `CPL_SPEC=1 CPL_STD=spec/std cpl check`. Pass -> `checks`.
4. Otherwise the fence must be marked `spec` explicitly, or the gate
   fails.
5. Write `web/.gate/status.json`; the site reads badges from it. A
   fence marked `runs` that stops running fails the gate.
6. Other-language fences (C++, Python, Java, Rust) in quickstarts and
   bug pairs are compiled or run with their toolchain.

## WASM

`crates/cpl-wasm` built with `wasm-pack` (or today's build in
`site/compiler.py`, *verified*) into `web/public/wasm/`, loaded by
`web/lib/wasm/worker.ts`. A parity test runs a corpus sample through
both WASM and native `cpl` (as `site/test_playground_parity.py` does
today).

## Tests

- Vitest: loaders, share-link encoding, problem judging logic.
- Playwright: landing renders and a hero tab runs; playground run,
  share, reopen; a course problem goes WA -> trace -> AC; search finds a
  library function; `/s` extracts source from a pasted file; offline
  reload of the playground.
- Lighthouse CI on three pages with the budgets in `README.md`.
- Link checker over the built site.

## CI and the repo gate

`scripts/check.sh --fast` gains a `web` step (pnpm install offline from
the lockfile, `pnpm gate`, `pnpm test`, `pnpm build`), replacing the
`site/` steps at cutover. Node and pnpm become required.

## Hosting

- Cloudflare Pages builds `web/out/` per branch (previews) and `main`
  (production).
- The Worker (`web/worker/`) handles `POST /api/share` and `GET /p/:id`
  (HTML shell + OpenGraph image) with R2 storage, rate limiting and
  size caps. `wrangler dev` runs it locally for tests.
- Until Ryan sets up the Cloudflare account, R2 bucket and domain,
  everything runs locally and nothing is deployed.

## Phases and their acceptance checks

0. **Skeleton**: `web/` with Next.js, shadcn, Fumadocs; the WASM worker;
   the gate; drafts of landing, one guide and one course problem.
   Check: `pnpm build` and `pnpm gate` pass; three pages previewed.
1. **Playground, landing, onboarding**: the editor component, `/play`,
   fragment share links, landing, getting started, four quickstarts,
   tour. Check: Playwright flows for each.
2. **Docs**: guides migrated to 0.3, the language, library and tools
   reference, cookbook, errors, cheat sheet, glossary, search,
   `llms.txt`. Check: every `spec/std` public function has a page entry;
   every error code has a page.
3. **Course**: format, judge, verdicts, tracks 1 to 3, then the rest.
   Check: every problem's CI checks pass.
4. **Project and setup**: status, design, decisions, real problems,
   performance, releases, contributing, setup, tools pages, Worker,
   `/s`, OG images. Check: all generated pages build from repo data.
5. **Cutover**: writable-from-docs on `/llms-full.txt` at least 49/55;
   `check.sh` gates `web/`; `site/` deleted. Check: the definition of
   done in `README.md`.

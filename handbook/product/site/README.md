# Site

The site is where most people meet CP Lang, and where many keep using it:
cp-lang.org in a browser, with the compiler running in the page. A
contestant should be able to go from "what is this?" to a solved problem
without installing anything, then install and keep the site open as the
reference beside their editor.

**Landing** answers "is this for me?" in one screen: a real problem solved
in a few lines next to its C++, a Run button that works, and the way to
install. It should take under a minute to decide to try it.

**Learn** is the course: chapters that teach the language in the order a
contestant needs it, and practice problems judged in the page. A reader
coming from C++ or Python should never hit a concept the course hasn't
introduced, and should be able to pass every problem in the browser.

**Reference** is the std library, the spec, the errors and the lints,
generated from the sources so they can't drift. You should find the item
for a task without knowing its name, and read its signature, options, cost
and edge cases at a glance.

**Playground** is the editor in the page: write, check, run, trace and
stress test a program, see the C++ it becomes, and share it as a link. It
should feel like a local editor with the language server, not a demo box.

**Practice** is the judge behind Learn's problems: hidden tests, verdicts
and hints, strict enough that a wrong solution never passes.

Each surface has its own doc and use cases: `playground.md` and
`playground-usecases.md` so far; the standing rules below hold for all of them. Owner: site. How the site is built and
checked is in `handbook/domains/site.md`.

## Standing rules

Each rule holds on every change; the check that enforces it is named.

- **Every landing example is a program.** The landing's tabs show programs with their input, and Run appears on each one the compiler runs; a tab group opens on one that runs (`runsFirst` in `web/lib/landing.ts`).
- **Guide code that runs is live.** Every guide block whose status is `runs` has Open in playground, and Run when it prints (`remarkLive` in `web/source.config.ts`).
- **One header.** Every page, on both Fumadocs layouts, uses `web/components/site-header.tsx`: title and links, then search, theme and GitHub; on phones, Install, search and one menu button.
- **Browser gates have no accepted findings.** a11y, search and completion parity fail on any finding; a fix lands with its test instead of a list entry (`scripts/debt-ratchet.py` counts any list that comes back).
- **Size caps only shrink.** `/` at most 800 KB gz with no wasm (`web/tests/page-weight.mjs`), the search index at most 500 KB gz, `cpl.wasm` at most 1,875,000 B gz (`web/scripts/wasm-size.mjs`).
- **Every prebuild stage declares its inputs.** A stage that reads a repo file it doesn't declare fails (`web/scripts/prebuild/stages.mjs`), so the cache never serves stale output.
- **Every LSP message goes through `lspTask`** (`web/lib/cpl.ts`), so a request never overtakes a `didChange` waiting on lazy std (`web/lib/lsp.test.ts`).

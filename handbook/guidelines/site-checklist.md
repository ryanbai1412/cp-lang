# Site pages checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | site |
| Explained in | `handbook/guidelines/site.md` |

Every rule a site page, the std reference rendering or a browser editor must follow. Each has a stable id that checks cite; `review` means nothing mechanical decides it yet.

## site/R1: Every program on a page is checked

Every `cp` fence and `.cp` file under `web/content` is checked, built and run, and its measured status (runs, checks, spec, rejects) must be at least what the page declares. Output shown on a page is the output of that run, and the browser build gives the same output and the same C++. Std doc `example:` lines run where they are rendered.

- **Why:** a reader who copies a program that does not work stops trusting every other page.
- **Example:** a guide prints `a.max()!`; when `max()` started returning an optional, the build failed on the guides that still printed `a.max()`, before a reader saw them.
- **Enforced by:** `web/scripts/gate.mjs`, `web/scripts/gen-ref.mjs`.

## site/R2: Each fact has one source

A fact that more than one page states (install commands, the version, std module names, retired spellings, spec parts) is read from one file or generated from the compiler, never retyped. A page that must mention it links to the source page.

- **Why:** a fact typed twice drifts, and the reader cannot tell which copy is right.
- **Example:** the install command lives in `web/lib/facts.json`; `/setup`, Getting started and `/llms-full.txt` read it, and the build fails on any other spelling of an install command in prose.
- **Enforced by:** `web/scripts/facts.mjs`.

## site/R3: Generated pages come from the compiler that ships

Never hand-edit a generated page or keep a stale copy; fix the generator or the source it reads.

- **Why:** a hand-edited reference disagrees with the compiler at the next change.
- **Example:** the std reference, lint pages, error pages and tool pages are rebuilt from `cpl doc`, `cpl lint --explain` and the diagnostics registry on every build; their cache is keyed on the `cpl` binary.
- **Enforced by:** `web/scripts/gen-ref.mjs`, `web/scripts/gen-tools.mjs`.

## site/R4: A reference entry reads as a structure, not a paragraph

Every std entry is rendered from the structured fields of `cpl doc --format json`; prose explains, and never restates a field.

- **Why:** a contestant scanning for one fact (what `by:` takes, what the cost is) should find it without reading a paragraph.
- **Example:** `runs` shows its signature as highlighted code, one parameter per line when it is long, then a table of its parameters with types and defaults (`by: T -> K = x => x`), then labelled rows for cost (as math), empty, order and traps, then examples that run.
- **Enforced by:** `web/scripts/gen-ref.mjs`. What std docs must contain is in `guidelines/std-docs.md`.

## site/R5: Every link is a real link

Anything that navigates is an `<a href>` or behaves like one: plain click navigates, Cmd/Ctrl-click and middle click open a new tab. Internal link targets and anchors exist.

- **Why:** contestants open docs beside their code; a link that cannot open in a new tab takes them away from their program.
- **Example:** Cmd+click (or a middle click) on a search result opens it in a new tab; Cmd+click on `SegTree` in any editor opens its reference entry.
- **Enforced by:** `web/scripts/links.mjs`, `web/tests/intelligence.mjs`.

## site/R6: Editors are the language server

The browser editor adds no analysis of its own; a feature the language server has is reached through it, and one it lacks is added there. Inlay hints and other optional aids have an off switch that is remembered.

- **Why:** two engines answer differently, and the browser one falls behind.
- **Example:** hover, completion, go to definition and inlay hints in the playground are requests to the same language server Neovim and VS Code use, compiled to wasm and called through `Playground::lsp`.
- **Enforced by:** `web/lib/lsp-parity.test.ts` (every capability the server advertises is used or a named gap in `web/lib/lsp-features.ts`), `web/tests/intelligence.mjs`.

## site/R7: Pages fit a phone and load light

No route is wider than 390 px; pages stay within the size budgets in `plan/domains/site/metrics.toml`; anything heavy loads on first use.

- **Why:** readers arrive from phones and slow links, and leave before a large page renders.
- **Example:** every route fits 390 px; the compiler (`cpl.wasm`) loads only when a reader runs or edits code, and Build, Explain and Stress load a second module on first use.
- **Enforced by:** `web/tests/phone.mjs`, `web/tests/page-weight.mjs`, `web/scripts/wasm-size.mjs`, `web/scripts/budgets.mjs`.

## site/R8: Design chapters cite the spec

A Language chapter explains a design against the alternatives it beat and links the spec section for the rule, instead of restating it. Every part of `spec/core` is cited by some chapter.

- **Why:** a restated rule drifts from the spec; the chapter's job is the why.
- **Example:** the Ownership chapter explains why `mut` borrows are explicit and links `R05` for the exact rule.
- **Enforced by:** `web/scripts/facts.mjs`.

## site/R9: User code runs only in the browser

Every check, run, trace, stress and build of a reader's program happens in the compiler's wasm worker in their browser; the site has no server-side compile.

- **Why:** a static site with no compile server cannot be overloaded or abused by the programs readers paste, and it works the same offline.
- **Example:** Run in the playground posts the program to the Web Worker that loaded `cpl.wasm`, which stops it after 10 s; nothing is sent to cp-lang.org.
- **Enforced by:** `web/lib/cpl.ts` is the only path to the compiler; review.

## site/R10: Share links never change

A share link is an immutable, versioned snapshot of at most 64 KB, so a link in an editorial still opens the same program years later.

- **Why:** people cite share links in discussions and editorials; a link that changes or breaks loses that record.
- **Example:** `/p/<id>` stores the source, input and version it was shared with; publishing again makes a new id.
- **Enforced by:** `web/worker/share.ts`, `web/worker/share.test.ts`.

## site/R11: Every layout shares one navigation

The header links and their order come from one list that every layout reads.

- **Why:** a reader who moves from Learn to the playground should find the same links in the same place.
- **Example:** adding Tools to the header is one entry in `web/lib/layout.shared.tsx`, and it appears on Home, Docs and Learn.
- **Enforced by:** `web/lib/layout.shared.tsx`; review.

## site/R12: An edit survives a reload

A reader's edit to any editor on the site, in the playground, an example or a practice problem, is still there after a reload, until they press Reset.

- **Why:** a contestant who reloads mid-solution, or comes back to a practice problem the next day, should not have to retype it.
- **Example:** fix the starter of a practice problem, reload, and the fix is still in the editor; Reset brings back the starter and forgets the edit.
- **Enforced by:** `web/lib/draft.ts`, `web/tests/drafts.mjs`.

## Changes

- 2026-11-23: first version, from the site guideline.
- 2026-11-23: site/R9 to R11 from the invariants in the former `plan/domains/site/ARCHITECTURE.md`.

## site/R13: A shared link previews its page

Every page the site exports carries its own title and description, and the site's image, as Open Graph and Twitter card tags, so a link pasted into a chat app or a social site shows what the page is.

- **Why:** contestants share a std page or a practice problem in a team chat; a bare URL with no title gets no clicks and looks broken.
- **Example:** a link to `/docs/ref/std/segtree` previews as "segtree · CP Lang" with the module's one-line summary and the CP Lang card.
- **Enforced by:** `web/scripts/social.mjs` (site postbuild).

## site/R14: Every page passes WCAG 2 A and AA

A route a visitor lands on has no axe-core violation of the WCAG 2 A and AA rules: text meets contrast, every input and editor has an accessible name, and every scrollable region is reachable by keyboard.

- **Why:** a contestant on a screen reader, on a dim screen at 3am, or driving the site from the keyboard during a contest should be able to read, search and run code.
- **Example:** each code editor carries an `aria-label` naming what it holds, so a screen reader announces "CP Lang source" instead of "edit text".
- **Enforced by:** `web/tests/a11y.mjs`, with today's violations in `web/tests/a11y-known.txt`, a list that only shrinks.

## site/R15: The browser's tools answer as the CLI's do

A tool the site runs in the page (Stress, and the checks and runs the gate already compares) gives the result its `cpl` command gives on the same program and options.

- **Why:** a contestant who stress tests in the playground and then locally must not get two different counterexamples, or a counterexample in one and none in the other.
- **Example:** for every repo program with an `@alt(program)` brute, the Stress tab and `cpl stress --explain` with seed 1, 30 cases and size 8 report the same verdict, shrunk input and first diverging variable.
- **Enforced by:** `web/scripts/stress-parity.mjs` (site prebuild); `web/scripts/gate.mjs` for check, run and C++.

## site/R16: The editor colours code as the grammar does

The browser editors' highlighter gives every word the class (keyword, literal, number, string, comment, other) the TextMate grammar gives it, and the grammar follows the compiler's keywords.

- **Why:** a word coloured as a keyword in the playground and as a name on a docs page, or the reverse, makes a reader doubt which it is.
- **Example:** `todo` is an ordinary name, so neither colours it as a keyword; `while:` as a named argument is a label in both.
- **Enforced by:** `web/components/editor/highlight-parity.test.ts` over `std/` and every port; `crates/cpl-cli/tests/vscode_sync.rs` keeps the grammar on the compiler's words.

## site/R17: A failed service degrades the page, never breaks it

When a service the site calls (the share API, a short link's lookup, the `cpl.wasm` download) fails, the page says what failed, keeps what still works and recovers when the service does.

- **Why:** a contestant mid-contest who can't share or whose compiler download dropped must still have their code, and a retry must work without a reload.
- **Example:** with the share API down, Share copies the full link, which needs no service; with `cpl.wasm` unreachable, Run says the compiler failed to load, and the next Run after it is reachable works.
- **Enforced by:** `web/tests/resilience.mjs` (site browser tests).

## site/R18: Search finds the item from the task

A contestant who types the task ("range minimum query", "union find") rather than the item's name finds the std module or topic page that answers it in the first five results.

- **Why:** a contestant who doesn't know the name yet is the one who most needs search; if it only matches names, they fall back to scrolling the module list.
- **Example:** "union find" finds `std/dsu`; "convex hull" finds the geometry topic page.
- **Enforced by:** `web/tests/search-tasks.mjs` (site browser tests); phrasings missed today are in `web/tests/search-known.txt`, which only shrinks.

## site/R19: Every practice problem has hints that point into the docs

Each practice problem's page holds hints inside its `<Judge>`, shown under **Hints** before the solution is accepted, and at least one links to a std page, a Learn chapter or a lint.

- **Why:** a learner stuck on a problem needs the next idea and where to read it, not the answer; a hint without a link leaves them guessing which page teaches it.
- **Example:** *At most* hints that the values at most x are a prefix, names `first_gt`, and links the Sorting and searching chapter.
- **Enforced by:** `web/scripts/gate.mjs` (site prebuild) fails on a problem whose hints link nowhere.

## site/R20: Unit coverage of site code never falls

`pnpm --dir web test` measures the statements, branches, functions and lines of `web/lib`, `web/components` and `web/scripts` its unit tests run, and fails below the floor in `web/vitest.config.mts`. The floor only rises.

- **Why:** most site code is checked only through browser tests and the build; without a floor, new logic lands with no unit test and the share that has one shrinks unseen.
- **Example:** a new parser in `web/scripts/` with no `*.test.mjs` lowers line coverage and fails the unit tests until it has one or the floor is lowered in review.
- **Enforced by:** `web/vitest.config.mts` coverage thresholds (site build's unit tests).

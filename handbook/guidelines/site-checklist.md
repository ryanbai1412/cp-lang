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

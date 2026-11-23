# RFC 0002: Product surfaces and use cases

| | |
|---|---|
| Owner | coordinator |
| Status | Draft: under discussion with Ryan |
| Ryan to review | yes |
| Changes | `handbook/product/`, new `handbook/process/product-design.md` and checklist, the domain charters |

## Summary

We decide what to build top down. Every part of CP Lang a user touches is a
*surface*. Each surface has a short doc saying how it should feel, and an
exhaustive list of *use cases*, each with a stable ID and a program that runs
in CI. The list is written once and then maintained. Gaps show up as unbuilt
use cases, not as user reports.

## Problem

Today we find frictions one at a time and fix them one at a time. A trial
turns up `.find()` with no index, `max()` sometimes returning `T` and
sometimes `?T`, and hover missing `by:`. Each gets fixed, but nothing would
have listed them in advance, so the next trial finds the next batch. We also
spend little time on what to build before building it.

## Proposal

### The hierarchy

Five levels. Each level is a refinement of the one above; the last is code.

1. **Journey**: a stage of a contestant's life with CP Lang
   (`handbook/product/journeys.md`, J1 to J10). Exists today.
2. **Area**: a group of surfaces that serve the same journeys.
3. **Surface**: one thing a user touches, with one owner.
4. **Use case**: one thing a user wants to do with that surface, with a stable ID.
5. **Acceptance program**: the canonical program or command for the use case,
   run in CI.

A sketch of the top three levels:

```text
std (J5, J6)                        owner: stdlib
  sequences      sort, search, find, scan, runs, windows, join
  sorted         SortedSet, SortedBag, SortedMap, cursors
  ranges         SegTree, Fenwick, Sparse, Lazy, Window
  graphs         Graph, dijkstra, bfs, flow, dsu
  trees          Rooted, up, lca, reroot
  strings        z, kmp, suffix array, hashing
  math           Mod, ntt, poly, matrix, primes
  grids          domains, Dir4, neighbours
  io             read, print, multitest, interactors
cli (J3, J6, J7, J9)                owner: tools
  check  run  test  build  stress  trace  explain  fmt  fix  new
editor (J5, J7)                     owner: tools
  completion  hover  signature help  go to definition  inlay hints  rename
  quick fixes  per-editor install (VS Code, JetBrains, Neovim)
site (J1, J2)                       owner: site
  landing  learn  reference  playground  practice
debugger (J7)                       owner: debugger
  traces  breakpoints  replay
performance (J8)                    owner: perf
  explain costs  profiles  judge profiles
```

A use case reads like this:

```text
### seq/U7: index of the first element that satisfies a condition
When: scanning input for the first position where a property starts to hold.
Program: let i = a.find_index(x => x > k)  // ?I, None if no element
Cost: O(n), stops at the first match. Misuse: a non-Bool closure is E0302 on the closure.
Status: gap
```

The acceptance program is the `Program:` line, completed into a runnable file
in the catalogue's code block, with its expected output.

### When we use it

- **Once, exhaustively:** every domain lists its surfaces and their use cases.
  The sources are journeys, how the 877 corpus ports actually use each
  surface, STL, Python, Rust and AtCoder Library equivalents, and textbook
  topic lists.
- **Then as living docs:** a change edits the use case it serves. Adding a
  std function doesn't touch the surface doc unless it adds a use case.
- **Feedback tests the map:** every friction is matched to a use-case ID. A
  friction with no ID is an *escape*, and gets added to the list. The escape
  rate measures how complete the lists are.

### What a design needs

Depth depends on how uncertain the decision is and how costly it is to
reverse, not on how much code it takes.

| Level | Example | Needs |
|---|---|---|
| A | A bug, or a use case already listed | Nothing extra |
| B | A new use case, std fn or CLI flag | The use-case entry |
| C | A new surface, or a model that spans surfaces (indices) | A brief: the ideal code, two or more alternatives, tried on many real programs, reviewed by another lane; Ryan reviews |

### How it stays true

- Every acceptance program runs in CI.
- Every public std fn names a use case it serves (`serves: seq/U7`).
- Every CLI subcommand and flag, and every LSP capability, appears in a catalogue.
- Every journey step has at least one use case.
- The number of `gap` use cases only goes down. IDs are never reused.
- The escape rate is reported hourly.
- Completion, hover and docs tests check the `Program:` spelling.

### Where it lives

- `handbook/product/surfaces/<area>/<surface>.md`: the surface doc, under one
  page of prose. It covers who uses the surface and when, how it should
  feel, the principles specific to it, alternatives rejected, and non-goals.
- `handbook/product/surfaces/<area>/<surface>-usecases.md`: the use cases, in
  the format above.
- `handbook/process/product-design.md` and its checklist: this process.
- `spec/` still says precisely what the language and std do. Use cases say
  what users need to do.

### Format

Concise. The surface doc is under a page; a use case is about five lines.
Writing rules are in `handbook/guidelines/writing.md`; a surface-doc
checklist covers the rest.

## Cost

About 1 to 2 hours per domain to draft its surfaces, then the cost of
editing one use case per change. We'll notice it's too much if lanes spend
more than 10% of their time on these docs.

## Alternatives

- **Keep fixing frictions as they come.** That is what we do now, and it
  finds problems only after users hit them.
- **A product requirements doc per feature.** It covers a feature but never
  the whole surface, so gaps between features stay invisible.

## Prior art

- **Rust RFCs:** a guide-level explanation comes before the reference.
- **Amazon's "working backwards":** write the user-facing announcement first.
- **Shape Up:** decide how much time a piece of work is worth before designing it.
- **Use-case catalogues and requirements traceability** in safety-critical
  engineering: every requirement has an ID and a test.

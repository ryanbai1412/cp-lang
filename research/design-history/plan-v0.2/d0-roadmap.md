# 13. Roadmap and milestones

Status: proposed. Each phase has exit criteria. Phases can overlap, but a
phase isn't done until its criteria are met, and correctness and
performance bars are never traded for breadth: a narrower working slice
beats broad half-working coverage.

## Phase 0: settle the design

- Finish the plan and get the user's review of the proposed items
  (decisions.md, key features).
- Exit: every row in decisions.md is decided or explicitly deferred.

## Phase 1: the Rust compiler, end to end

The Python prototype in `proto/cpc/` proved the core ideas; the real
compiler is written in Rust.

- The full language of docs 1.1-1.9: types and range types, domains and
  bounds, optionals, records, structs and enums, traits, operation values and
  generics, generators and views, ownership and `mut`, program structure,
  typed I/O, with every check and diagnostic the docs describe.
- Both ways to run a program: compiled to one C++ file, and the debug
  interpreter.
- Exit:
  - the 1.1 tour program and 20 corpus ports compile and run on both
    backends with identical output;
  - interpreter vs C++ differential tests and the program fuzzer run in CI.

## Phase 2: ports and the core stdlib

- List, Map, HashMap, Heap, sort and search, SegTree, LazySegTree,
  Fenwick, SparseTable, DSU, ModInt, graphs and trees, strings basics,
  sequence views, reductions.
- Port 100+ corpus problems by rethinking, each with a generator and
  differential tests against the original (doc 8).
- Fix friction in the design as it shows up, logging each change.
- Exit: 100 ports meet the four bars of doc 8; the `sizeof` and assembly
  checks pass; the bug corpus is at 90% or more.

## Phase 3: tooling for daily use

- `cp run` with `i1`-`i5`, `cp test`, `cp stress` with smart shrinking,
  `cp build` with judge profiles and the size check, `cp new` and
  `cp fetch`, `cp fmt`, `cp lint`, version stamps, `lib/` snapshots and
  upgrades.
- The HTML trace with filtering, and snapshots that rerun one part.
- VS Code: TextMate highlighting, then diagnostics, hover and formatting
  through the LSP.
- Exit: compiles under 500 ms on every port; a full virtual contest solved
  using only these tools.

## Phase 4: the full experience

- Debugger over DAP with time travel, live sample values, cost warnings,
  profiler, coverage, bench, interactors and checkers, refactorings.
- Exit: the user reports less friction than C++ across a virtual contest
  set.

## Phase 5: docs and public

- The docs site, error index and cookbook (doc 9), the playground with the
  learning path, `llms.txt`, `cp port`, all judge profiles tested in CI.
- Exit: the user uses it in rated contests by default; publishing happens
  only through official channels with the user's approval.

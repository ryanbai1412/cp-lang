# 0. Overview

CP Lang is a language for competitive programming that compiles to one
C++17 file. Its mission is to minimize the time from an algorithmic idea
to an accepted submission the contestant understands and trusts
(`AGENTS.md`). This page is the model in one place; every claim points
to the chapter that defines it.

## The model

- **Types carry the facts a contestant already knows.** A domain
  `domain V = 1..=n` is a fresh index type, so vertices, edges, rows and
  columns cannot be mixed up; a range type `1..=1e9` is a contract that
  gives storage width, overflow proofs and input validation; `?T` is
  "maybe absent" with no sentinel and usually no extra bytes. (chapters
  2-4)
- **Values are owned or lent.** Heap values move, copies are
  `.clone()`, parameters are lent, `mut` marks mutation at both ends,
  and a lent value cannot outlive or race its source. There are no
  lifetimes in signatures except `lends`. (chapter 5)
- **Sequences are generators and views.** A generator pushes elements to
  a loop body and allocates nothing; views (slices, `filter`, `map`)
  keep their source's keys; library operations ask for the weakest of
  six capabilities. (chapters 6-7)
- **Operations are values.** `+`, `max`, `gcd` and user monoids are
  values with laws; segment trees, sparse tables, sliding windows, folds
  and convolutions take them and pick a documented specialization from
  the laws. (`lib/ops-ranges.md`)
- **The stdlib has no privileges.** Everything it does, a user library
  can do; the list of powers is chapter 12.
- **Costs are visible.** No-parenthesis members are O(1); anything that
  allocates, copies or traverses looks like it; `cpl explain` shows
  every implicit choice (widths, layouts, specializations, copies).
- **Two builds, one meaning.** Checked builds trap on every unproven
  fact; release builds run no checks by default and never contain C++
  undefined behaviour. Valid programs behave identically in both and in
  the interpreter. (chapter 9)

## A small program

```
domain V = 1..=read<1..=2e5>()
with io(base: 1)
let m = read<0..=2e5>()
let g = Graph.undirected(V, read<[m](u: V, v: V, w: 1..=1e9)>())
let p = dijkstra(g, from: [(V.first!, 0)])
print(p.dist[V.last!] ?? -1)
```

Every fact is stated once: the vertex count bounds positions (4-byte
storage), 1-based input converts at the reads, weights are validated and
proven not to overflow a path sum of `2e5 * 1e9`, and an unreachable
target is `none`, not `INF`.

## Layers

- **Core**: syntax, types, ownership, control flow, traits, contexts,
  checks and lowering (chapters 1-13).
- **Library**: `std/*.cp` signatures with documented costs, and the
  family chapters in `lib/`.
- **Tools**: the `cpl` CLI, judge profiles, stress and shrinking,
  explain, trace and debugger, fmt and lint, LSP, bundler (`tools/`).
  A feature is not done until it works in every tool.

## Vocabulary

| Word | Meaning |
|---|---|
| domain | a fresh index type with a count (`V`, `t.Pre`, `Dir`) |
| position | an element index of a domain (`v: V`) |
| cut | a boundary between positions (`V.cuts`, `Cut<V>`); `Int(c)` counts positions before it |
| range | `a..b` of positions or cuts; also a range type `lo..=hi` of integers |
| key | what a sequence is indexed by; slices keep source keys |
| view | a sequence built on another without copying (slice, `filter`) |
| generator | a recipe that pushes elements to a loop body |
| lend | a borrow of a value for as long as the borrower is used |
| niche | a spare bit pattern of a type that encodes `none` or an enum tag |
| law | a promised property of an operation, sampled by stress |

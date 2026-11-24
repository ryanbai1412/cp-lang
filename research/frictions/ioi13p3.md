# Friction Report: ioi13p3

## 1. Stateful `@grader` object (IOI interactive/grader API)

- **Area:** core
- **Goal:** express
- **Severity:** medium
- **Dream:** `proto/ports/scout_batch_52/dream/ioi13p3.cp:46-79`
- **Port:** `proto/ports/scout_batch_52/ioi13p3.cp:48-84`

IOI grader problems expose multiple entrypoints (`init`, `changeH`, `changeV`, `escape`) sharing mutable state whose domains (`Block = 0..ceil_div(r, B)`) depend on arguments passed to `init`. Today CP Lang has no `@grader struct` state object, so runnable ports use a stdin command loop instead of `@grader` methods.

## A block of a domain is spelled with cuts
- **Where:** `block`: `span(Row.cuts.nth(b * BLOCK), Row.cuts.nth(min(r, ...)))`.
  `Row.all[lo..hi]` does not slice a range, and there is no `Row.chunks(BLOCK)`
  domain of blocks to index the SegTree with.
- **Severity:** medium.

## Neighbouring columns of a gap need `Col(g.ord + 1)`
- **Where:** the east/west relaxations: `Gap` sits between `Col`s, but nothing
  relates them, so each step converts through `ord`. A `Gap = Col.gaps` (the
  cuts strictly inside) with `.left`/`.right` would type it.
- **Severity:** low.

## "Better than an optional best" is `map(..) ?? true`
- **Where:** `mul`: `out[i, j].map(best => via < best) ?? true`. Storing
  `out[i, j] min= Some(via)` loses which `k` won, which Knuth's bound needs; a
  `min_by_key` over `lo..=hi` returning the key would do both.
- **Severity:** low.

## `?Cost` cells take 8 bytes, so blocks are 20 rows, not 10
- **Where:** `const BLOCK = 20`: a `[Col, Col]?Cost` emits `Grid<OptInt>`
  although `?0..=1e7` fits `OptI32`, so 10-row blocks reach 486 MB at the
  max case (C++: 179 MB). 20-row blocks halve the tree to 248 MB, under the
  256 MB limit. Repro sent to Core.
- **Severity:** medium.

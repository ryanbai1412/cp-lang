# Friction report: ioi25_obstacles

## Dream

```cp
for r in t.record_highs():                       // rows with a new maximum temperature
    let opened = waiting.pop_front_while(c => h[c] < t[r])
    for (id, run) in [(c, run_around(c, t[r])) for c in opened].distinct(by: p => p.1.first!):
        layer[id] = Some(Layer(r, run))
        for (_, child) in open.drain(after: run.l.l, while: (c, _) => c in run):
            ...
let layers = Lift(up, tightest, identity: Drop(Col.first!, Col.last!))
```

Both the port and the dream are this layer forest with binary lifting, the same algorithm as
the official model solution. Both are stdin programs that read the sample grader's input,
call the setup once and print one `can_reach` per query.

## A stateful `@grader mod` with domains sized by `initialize` doesn't compile
- **Where:** `dream.cp` as first written: `@grader mod obstacles:` with `domain Row, Col //
  sized by initialize`, `var layers: Lifted<Layer, Drop>` and `var home: [Col]?Node` set in
  `fn initialize(t: [Row]0..=1e9, h: [Col]0..=1e9)` and read in `fn can_reach`. A fresh
  `cpl check` reports ``expected '=' and the domain's range ... found ','``, then a `var`
  without a value. Giving each domain its own line doesn't help: a module-level domain has no
  size until `initialize` runs. Storing the lent parameter fails with ``error[E0603]: lent
  parameter `h` cannot be stored in an outer var``.
- **What felt wrong:** the IOI interface is one setup call and many queries sharing state
  whose shape the setup decides. CP Lang can't state that, so the dream copies the sample
  grader's I/O loop. The program verified is then not the program submitted, the same gap as
  research/frictions/ioi13p3.md and the `@grader` section of
  research/frictions/ioi25_souvenirs.md.
- **Goal it hurts:** express, submission.
- **Ideal code:** the dream as first written: `@grader mod` whose `domain Row, Col` are bound
  by the first entry's parameter types, `var`s that start unset and are assigned once in
  `initialize`, and parameters copied into the state.
- **General fix (compiler/spec):** grader-module state: domains bound by the setup entry's
  parameters, `var` fields that are definitely assigned by the setup before any other entry
  reads them, and an owned copy when a lent parameter is stored. This would cover every
  IOI init-plus-query task (ioi13p3, ioi25_*, ...).
- **Severity:** medium.

## Std names the dream wanted that don't exist
- **Where:** `dream.cp` as first written. Each name, and what the dream uses now:

| Wanted | Used instead |
|---|---|
| `Forest<Layer, Drop>()` with `.add`, `.link(child, parent:, drop)`, `forest[v]`, and `.lift(tightest, identity:)` giving a `Lifted<Layer, Drop>` | layers named by the column that opened them: `var layer: [Col]?Layer`, `var up: [Col]?(Col, Drop)`, then `Lift(up, tightest, identity:)` from `std.tree` |
| `RangeMin(t)`, `RangeMax(h)`, indexed `coldest[a..=b]` | `SparseTable(min, t)`, `SparseTable(max, h)`, `.fold(a..=b)` |
| `driest.first(in: cols, where: x => x < warm)` / `.last(...)` | `cols.first_true(c => driest.fold(cols.first!..=c) < warm)`, the same with `last_true` |
| `wettest.span(around: c, while: x => x < t[r])` | a local `fn run_around(c, limit)` built from two `first_true`/`last_true` searches over `wettest.fold` |
| `open.drain(in: run)` | `open.drain(after: run.l.l, while: (c, _) => c in run)` |
| `Queue(Col.sort(by: h))` and `.pop_while` | `Deque.of(Col.all.sorted(by: c => h[c]))` and `.pop_front_while` |

- **What felt wrong:** the replacements state how to search, not what is wanted. `drain(after:
  run.l.l, ...)` needs the cut before the run, and the first dream that compiled wrote
  `after: run.first!`. That compiled and silently skipped the child layer that starts at the
  run's first column (wrong answers on a 4 x 4 grid; the fix is
  `run.l.l`). `drain(in: range)` can't be written that way.
- **Goal it hurts:** express, hard-to-get-wrong.
- **General fix (stdlib):** `SortedMap.drain(in: Range<K>)`. `SparseTable.first(in:, where:)`
  / `.last(in:, where:)` and `span(around:, while:)` for monotone predicates over a range
  fold. A `Forest` builder whose `.lift` returns `Lift`. Owner: stdlib lead (Content).
- **Severity:** medium (the drain bug), low (the rest).

## `distinct(by:)` on a `Range` key makes C++ that doesn't compile
- **Where:** `dream.cp`, `.distinct(by: p => p.1)` where `p.1: Range<Col>`. `cpl check`
  passes, but g++ fails on the emitted `cpl::HashSet<cpl::Range<int>, cpl::CpHash>::insert`
  with `no match for call to '(cpl::CpHash) (const cpl::Range<int>&)'`: `CpHash` has
  overloads for integers and strings only. The dream now keys on `p.1.first!` (runs from the
  same row are disjoint, so the first column names a run).
- **What felt wrong:** a program the checker accepts failing in g++ is a codegen bug. Nothing
  in the source shows that a `Range` can't be a hash key.
- **Goal it hurts:** hard-to-get-wrong, submission.
- **General fix (codegen):** emit `CpHash` for every type the checker accepts as `Hash`
  (ranges, tuples, structs), or have the checker reject the key. Owner: codegen.
- **Severity:** medium.

## The port's C++ is 45 KB, over the 32 KB target
- **Where:** `cpl build --judge codeforces port.cp --size`: 44993 bytes. Runtime 19859 (45%:
  hashcore 3290, cphash 1814, cphashmap 1470, ...), library fns 14356 (32%: `Lift.new`
  1466, `SparseTable.new` 1146, `SortedBag.erase_at` 930, ...), your code 4804 (11%).
  Replacing `distinct(by:)` with a check on `open` saves only 1.6 KB (43417 bytes): the hash
  runtime is still pulled in. `scripts/corpus.py verify ioi25_obstacles` reports
  `over-32kb ioi25_obstacles 44993`.
- **Goal it hurts:** submission.
- **General fix (codegen):** emit only the runtime the program reaches, and only the fields of
  `Lift`/`SparseTable`/`SortedMap` that it uses.
- **Severity:** low (69% of the 64 KB Codeforces limit).

## Measurements

- **Tokens:** port 713, dream 713 (both include the stdin reading), original (model solution,
  sample grader main excluded) 1814.
- **Runtime and memory:** on the `--max` case (N = M = Q = 2e5), port 0.45 s / 149 MB,
  `orig.cpp` 0.62 s / 81 MB, with identical output.
- **Emitted C++:** port 44993 bytes, dream 44920 bytes.
- **Agreement:** `orig.cpp` (the official model solution) passes all 76 official tests and
  matches a BFS brute on 800 random grids. The dream matches it on the 76 official tests, the
  14 package cases (`dream_stress`) and 1500 more gen.py seeds.

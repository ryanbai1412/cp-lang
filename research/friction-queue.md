# Friction queue

This is the one queue of language and library friction found while porting
contest problems to CP Lang 0.2.2. It replaces reading the scattered
`// WISH` comments and backlog files one by one. Every open gap below is
either a general language mechanism or a library abstraction that several
problems share; one-problem helpers are listed at the end as rejected, so
nobody builds them by accident.

## Where the items come from

There are 363 `// WISH` markers in 130 files under `proto/ports/`.
Most name a code such as `G16` or `TR4`; that code is a heading in the
matching `research/friction-backlog-batch-*.md`, and the clustering below
read the heading as well as the comment. The other source is the
35 files in `NOT_BUILT` in `crates/cpl-cli/tests/corpus.rs`: ports
and prototype tests that the Rust compiler does not check clean yet.

Every claim tagged **[ALREADY SHIPPED — port cleanup]** was checked by
running a small program with `cpl run` on `main` at `602aa80`. Those items
need the ports' `// WISH` comments and workarounds removed, not compiler
work. **[OPEN]** items failed on `main`; the error is quoted.

Each marker sits in exactly one cluster (the first one whose words it
matches, in the order of the appendix), so the counts add up to 363:

- Ranges, slices and sequence adapters: 39 markers in 26 ports.
- Grids, directions and 2-D views: 37 markers in 18 ports.
- Typed inputs for `cpl stress`: constraints a type cannot say: 36 markers in 33 ports.
- Numbers: Mod, wide ints, bits, polynomials, floats: 34 markers in 23 ports.
- Long tail (single-problem wishes): 32 markers in 22 ports.
- Output formatting: 30 markers in 20 ports.
- `read()` formats: 26 markers in 23 ports.
- The search family: 21 markers in 20 ports.
- Optionals and patterns: 21 markers in 21 ports.
- Records, enums and methods: 18 markers in 14 ports.
- Judging: checkers, interactors and graders: 17 markers in 13 ports.
- Trees: 15 markers in 12 ports.
- Range structures and monoids: 13 markers in 13 ports.
- Maps, sets, heaps and keys: 13 markers in 9 ports.
- State: memo tables, undo, pools, cursors: 8 markers in 6 ports.
- Graphs: 3 markers in 3 ports.

The full list of markers by cluster is in the appendix.

## Already shipped: port cleanup only

These wishes are answered on `main`. A port that still carries the
`// WISH` comment, or the workaround next to it, should be rewritten to
use the shipped form.

**`if let` and `while let` as expressions** [ALREADY SHIPPED — port cleanup].
`btoi15p2` (C2) and `ceoi22p3` (TR17) say `if let` is a statement only
and that `while let x = stack.pop()` is missing. Both run:

```cp
print(if let t = below(i, 1): ops[t] else: 0)
while let x = stack.pop(): visit(x)
```

**`all`, `any` and `count` in the prelude** [ALREADY SHIPPED — port cleanup].
`cco20p4` and `btoi18p2` (C1) say these are on the cheat sheet but not in
the prelude. `print(all(x > 0 for x in xs), count(need[s] > 0 for s in Sym))`
runs without a `use`.

**The search family** [ALREADY SHIPPED — port cleanup]. `ioi09p7` and
`mmcc14p3` (TR4) wish for `bsearch_first`. It and `bsearch_last` are in
the prelude, and a sorted list has `first_ge`:

```cp
let r = bsearch_first(0..n, r => outposts(r) <= k)
let i = ws.first_ge(p)
```

**A queue** [ALREADY SHIPPED — port cleanup]. `ccc18s3` (L4, "no queue")
writes `var head = 0; while head < q.len`. Use
`var q = Deque.of(starts)` and `while let u = q.pop_front()`, or `bfs` and
`peel` from the prelude.

**Fixed decimals, lists and records in `print`** [ALREADY SHIPPED — port cleanup].
`dpi` and `noi07p1` (MG9) wish for `{ans:.10f}`; `ceoi22p3` (TR15) wants a
list on one line; `yac6p3` wants a record to print its fields.
`print("{x:.10f}")` prints `1.5000000000`, `print([1, 2, 3])` prints
`1 2 3`, and `print(P(1, 2))` prints `1 2`.

**`Mod<P>` literals, compound assignment and input** [ALREADY SHIPPED — port cleanup].
`noi19p5` (MG4) asks for `Int` to `Mod<P>` promotion so `Z(k)` disappears,
and `dpr` (G14) for reading straight into `Mod`. All of these run:

```cp
let x: Mod<7> = 3
var y = x + 5
y *= 3
let z: Mod<P> = read()
```

**Numbers past `Int`** [ALREADY SHIPPED — port cleanup]. `modinv` (G9)
wants wide intermediates: `I128` and `U64` are built-in types.

**Slices and open ranges** [ALREADY SHIPPED — port cleanup].
`xs[..2]`, `xs[1..]` and `for i in 1..:` all run.

**Reading a string into a domain array** [ALREADY SHIPPED — port cleanup].
`noip19p2` (TR9) says `let s: [V]Char = read()` reads numbers. On `main` it
reads `abc` as three characters.

**Trees from a parent list, and post-order** [ALREADY SHIPPED — port cleanup].
`ioi09p7` and `noip19p2` (TR3) build child lists by hand to pass to
`Rooted.new`. `Rooted.from_parents(par)` takes the `[V]?V` directly.
`ccc22s5` (W20) wants `postorder()`: `for p in t.Pre.rev: f(t.at[p])`
already visits children before parents.

**Order statistics** [ALREADY SHIPPED — port cleanup]. `ioi12p6` (C7) and
`ccc05s5` (F13) want a bag with rank queries: `SortedBag<K>.of(xs).rank(x)`.

**Optional arithmetic** [ALREADY SHIPPED — port cleanup]. `ioi04p2hard`
(X1) wants arithmetic on an optional: `some(3) + 2` is `some(5)`, and
`a min= 4` on a `none` gives `4`.

**Grid moves** [ALREADY SHIPPED — port cleanup]. L1 ("moving on a grid", 6
of 10 in batch 12) and L2 (a direction from the input) are `std.grid`:

```cp
use std.grid.{Dir4, step, adj4}
let d = Dir4.from("UDLR", ch)!
if let q = p.step(d): go(q)
for q in p.adj4: relax(q)
```

**Topological order with cycles** [ALREADY SHIPPED — port cleanup].
`noi09p4` (W16) says `topo` gives `none` on any cycle. `peel(g)` returns
the peeled order and leaves out the vertices that reach a cycle.

**Others that run on `main`**: `match` as an expression (G19), `dedup` and
`rotate_left` in `std.seq` (W8, C3), `@!multitest`, `popcount` / `ctz` /
`bit_width` (part of P11), `LazySegTree.max_right` (W9; it takes a
`Bounds`), and ports using the standard library at all (F1, batches 1 to
13).

## Open gaps, by cluster

Each cluster lists what failed on `main`, with the error. The clusters
with no open item (search, output, trees, graphs) are not repeated here.

### `read()` formats (26 markers, 23 ports)

`read()` covers scalars, tuples, `Str`, `Mod`, and `[D]T` for a domain
`D`, but not the rest of the 7 auto-derived `Read` types. This is the
largest cluster, and 5 `NOT_BUILT` files fail only on it.

- `let b: Bool = read()` fails: "`read()` has no input format for a `Bool`" (`dpo`).
- `let p: [V]?V = read()` fails: "`read()` has no input format for a `?V`" (`yac6p2`; TR3 parent lists with a `-1` or `0` root).
- `let xs: List<Int> = read()` fails: "`read()` has no input format for a `List<Int>`" (`mccc4j5`; the count-then-items line).
- Reading an enum written as letters fails: "reading a `Op` needs to know how its variants are written in the input" (`noi02p1`, A4, L2, L26). With `@tags("A") Add` on a variant, the checker reports "no `Add` in scope".
- `read(none: "X")` fails: "`read` has no argument named `none`" (B19-6, L20: a cell read from `.` or `0`).
- `read_line()` fails: "no `read_line` in scope" (B19-9, `noimps22p1`).
- The old spelling `read(n)` is used by `cco18p1` and `ergo/columns`: "`read` takes 0 arguments, but 1 was given". That is port cleanup, not a gap.

### Typed inputs for `cpl stress` (36 markers, 33 ports)

Statements promise things a type cannot say: a sorted array, a
permutation, a connected graph, a tree, primes, `n` even, or one input
bounded by another (S4, R5, K19, W4, W21, MG11, A21, B19-3). The
general mechanism already exists: `read(gen: ...)` with a typed generator,
plus `@gen`. The gap is a small set of generator combinators in the
library, not new syntax: `sorted`, `distinct`, `perm`, `tree`,
`connected`, and a `such that` filter with shrinking. These belong in a
library batch, not a syntax batch.

### Grids, directions and 2-D views (37 markers, 18 ports)

`step`, `ray`, `adj4` and `Dir4` ship. What is still open:

- Finding a cell: `g.find(c => c == 'S')` on a `[R, C]Char` fails: "a `[R, C]Char` has no method `find` that `cpl run` knows" (L6, 3 of 10).
- `Dir8` does not exist: "`std.grid` has no `Dir8`" (L1 over `Dir8`, `coci10c7p4`). `adj8` exists, so this is one more direction type.
- Mirrored and turned views (`G2`, `L22`, `X6`: 4 ports) are wanted as values. `transposed` exists; a reversed view of a domain is the general form.

### Optionals and patterns (21 markers, 21 ports)

- `let (x, y) = (a, b) else: ...` on two optionals fails: "`let ... else` takes the value out of an optional, but this is a `(?Int, ?Int)`" (F1 in batch 44, `ergo/sugar`, `apio07p2`). A tuple pattern over optionals is the general fix.
- A fieldless enum as an index domain fails: `[S] s => 1` gives "`S` is a type, and a value is needed here" (P5, W19). The README says enums implement `Index`.
- `return` inside a `do:` block used as a value fails: "`cpl run` and `cpl build` can't handle `return` inside a block that is used as a value yet" (`ok/do2`).

### Ranges, slices and sequence adapters (39 markers, 26 ports)

- `zip(a, b)` fails: "no `zip` in scope" (G21).
- `xs.pairs()` for consecutive pairs fails: "a `List<Int>` has no method `pairs`" (L18, F11).
- `xs.cycle` fails (G21). With `take`, these three are the missing iterator adapters, and they are one cluster in `Iter`.
- Intersecting two half-open ranges is written with `max` and `min` by hand (X5, A15). `Range.meet` would be one general operation.

### Numbers (34 markers, 23 ports)

- `clz` fails: "no `clz` in scope" (P11). `popcount`, `ctz` and `bit_width` ship.
- Submask iteration: `for sub in submasks(m)` fails: "no `submasks` in scope" (MG8, P10). `Subset<D>` is the general place for it.
- `product` fails: "no `product` in scope" (`dpp`, `noi01p5`). It is the multiplicative twin of `sum`.
- SOS / zeta-Mobius transforms over `Subset<D>` (MG7, P10) and `Poly` with `inv`, `eval` and `interpolate` (MG6) are library work in `std.poly`.
- `sign(x)` fails (G20). It is a single port, so it is rejected below.

### Judging (17 markers, 13 ports)

Checkers that call `read_output()` on variable-size output (B19-7), a
grader ABI with `extern fn` / `export fn` (B19-4, B19-5), and the question
budget next to `ask` (K21, R10). These are tooling work on the
`@checker` / `@interactor` path, not syntax.

### Maps, sets, heaps and keys (13 markers, 9 ports)

- `Set.of(xs)` fails in `cpl run`: "`cpl run` doesn't know the name `Set` here"; a set comprehension does not parse (G5).
- `a | b` on two sets fails: "`|` can't be applied to `Set<V>` and `Set<V>`" (`ok/subset2`).
- `max(xs, by: f)` fails: "`max` has no argument named `by`" (`ioi13p1`). `min_by` / `max_by` exist in `std.seq`, so the keyword form on the prelude reductions is one consistent spelling.

## Ready-to-run syntax batches

Each batch is general mechanisms only, small enough for one worker, with
the ports it unblocks.

### Syntax Batch #3: `read()` for every `Read` type

1. `Bool` (read `0`/`1`), `?T` with `read(none: "X")`, `List<T>` (count, then items) and fieldless enums with `@tags`, all through the one `Read` trait rather than per-type cases in the checker.
2. `read_line()` that keeps spaces.

Before and after, from `yac6p2` and `noi02p1`:

```cp
// before
let raw: [V]Int = read()
let boss: [V]?V = [V] v => if raw[v] == 0: none else: some(V.of(raw[v] - 1))
// after
let boss: [V]?V = read(none: "0")

// before
let tag: Str = read()
let op = if tag == "A": Op.Add else: Op.Sub
// after
enum Op:
    @tags("A") Add
    @tags("S") Sub
let op: Op = read()
```

It unblocks `dpo`, `yac6p2`, `mccc4j5`, `noi02p1` and `coci20c6p3` in
`NOT_BUILT`, and the 26 `read()` markers.

### Syntax Batch #4: optionals, enums and patterns

1. Tuple patterns over optionals in `let ... else` and `if let`.
2. Fieldless enums as index domains, `[S] s => ...` and `for s in S`.
3. `return` inside a `do:` value block.
4. `Set.of` and set comprehensions in `cpl run`, and `|`, `&`, `-` on sets.
5. `by:` on the prelude reductions `min` and `max`, as `sort(by:)` already has.

Before and after, from `ergo/sugar` and `ccc22s5`:

```cp
// before
let a = left(v) else: continue
let b = right(v) else: continue
// after
let (a, b) = (left(v), right(v)) else: continue

// before
var best: [V, 0..3]?Int = none
// after
enum State:
    Free
    Held
    Done
var best: [V, State]?Int = none
```

It unblocks `ergo/sugar`, `ok/do2` and `ok/subset2` in `NOT_BUILT`,
plus the 21 optional-and-pattern markers.

### Syntax Batch #5: sequences, grids and bits

1. `zip`, `.pairs()` and `.cycle` on `Iter`, next to the existing `take`.
2. `find` on `[D]T` and `[R, C]T` that returns `?D` / `?(R, C)`.
3. `Dir8` in `std.grid`, mirroring `Dir4`.
4. `clz` next to `ctz`, `submasks` on `Subset<D>`, and `product` next to `sum`.

Before and after, from `ccc18s3` and `noimps22p4`:

```cp
// before
var start = (R.first, C.first)
for r in R:
    for c in C:
        if g[r, c] == 'S': start = (r, c)
// after
let start = g.find(c => c == 'S')!

// before
var sub = m
while sub > 0:
    f(sub)
    sub = (sub - 1) & m
// after
for sub in submasks(m): f(sub)
```

It unblocks `dpp` and `noi01p5` in `NOT_BUILT`, plus the grid, sequence
and number markers above.

## NOT_BUILT: the first error on `main`

`cpl check` on each file in `NOT_BUILT`. **Port cleanup** means the port
uses a name or spelling that 0.2.2 renamed or never had. **Gap** means it
waits on one of the batches above, or on a feature named in the error.

- `ports/ccc03s4.cp`: [E0201]: no `StrHash` in scope — port cleanup: `StrHash` became `RollingHash` / `HashVal`.
- `ports/ccc20s3.cp`: [E0201]: no `StrHash` in scope — port cleanup: `StrHash` became `RollingHash` / `HashVal`.
- `ports/ccc23s4.cp`: [E0303]: `dijkstra` takes 3 arguments (next, from, w), but `w` is missing — port cleanup: `dijkstra(next, from, w)` takes the weight function.
- `ports/cco18p1.cp`: [E0303]: `read` takes 0 arguments, but 1 was given — port cleanup: the old `read(n)` spelling.
- `ports/ceoi09p3.cp`: [E0302]: a `Compressed<D_1, Int>` can't be taken apart into 2 names — gap: `compress` returns a record that a tuple pattern cannot split.
- `ports/coci20c6p3.cp`: [E0501]: `read()` can't tell how many values a `[W]Letter` has — gap: Syntax Batch #3.
- `ports/dpr.cp`: [E0302]: expected `[V, V]Mod<_>`, found `[V, V]Mod<1000000007>` — gap: a generic `Mod<_>` does not unify with a fixed modulus.
- `ports/hillclimb_dp_graph/dpo.cp`: [E0501]: `read()` has no input format for a `Bool` — gap: Syntax Batch #3.
- `ports/hillclimb_dp_graph/dpp.cp`: [E0201]: no `product` in scope — gap: Syntax Batch #5 (`product`).
- `ports/hillclimb_dp_graph/noi15p2.cp`: [E0303]: `AssignSum<tree.Pre>` takes 4 arguments (id, compose, apply, over), but `id`, `compose`, `apply`, `over` a — port cleanup: `AssignSum` is a ready `Action` value, not a constructor.
- `ports/hillclimb_dp_graph/yac6p2.cp`: [E0501]: `read()` has no input format for a `?V` — gap: Syntax Batch #3.
- `ports/ioi11p2.cp`: [E0310]: `Rooted` has no parameter named `weight` — port cleanup: `Rooted.weighted(adj, root, len)`.
- `ports/mccc4j5.cp`: [E0501]: `read()` has no input format for a `List<V>` — gap: Syntax Batch #3.
- `ports/nccc10s2.cp`: [E0204]: no type or domain named `Perm` — port cleanup: a permutation is a `[V]V` (Index unification).
- `ports/noi01p5.cp`: [E0201]: no `product` in scope — gap: Syntax Batch #5 (`product`).
- `ports/noi02p1.cp`: [E0501]: `read()` has no input format for a `Op` — gap: Syntax Batch #3.
- `ports/yac1p1.cp`: [E0302]: expected `Int`, found `Rank` — gap: see the error.
- `tests/ergo/columns.cp`: [E0303]: `read` takes 0 arguments, but 1 was given — port cleanup: the old `read(n)` spelling.
- `tests/ergo/counter.cp`: [E0201]: no `StrHash` in scope — port cleanup: `StrHash` became `RollingHash` / `HashVal`.
- `tests/ergo/effects_first.cp`: [E0304]: `heap.len` is an Int, not a fn — gap: see the error.
- `tests/ergo/moves.cp`: [E0302]: this needs a sequence of values, not a `Int` — gap: see the error.
- `tests/ergo/prefix.cp`: [E0501]: `cpl run` and `cpl build` can't handle `prefix` without `stdlib/prefix.cp` yet — gap: `prefix` has no lowering in `cpl run`.
- `tests/ergo/ranges.cp`: [E0302]: `B` is a type, and a value is needed here — gap: see the error.
- `tests/ergo/sugar.cp`: [E0302]: `let ... else` takes the value out of an optional, but this is a `(?G, ?G)` — gap: Syntax Batch #4.
- `tests/fn/invariant.cp`: [E0501]: `cpl run` and `cpl build` can't handle the `@invariant` decorator yet — gap: `@invariant` has no lowering.
- `tests/lend/grid_rows.cp`: [E0203]: a `[Row, Col]Char` has no method `row` that `cpl run` knows — gap: see the error.
- `tests/lend/projection.cp`: [E0401]: only a variable, a field, or an element can be assigned to — gap: see the error.
- `tests/lend/slices.cp`: [E0301]: this is indexed by `V`, but the index is a `Range<V>` — gap: see the error.
- `tests/ok/at1.cp`: [E0203]: a `I` has no member `mirror` that `cpl run` knows — gap: see the error.
- `tests/ok/do2.cp`: [E0501]: `cpl run` and `cpl build` can't handle `return` inside a block that is used as a value yet — gap: Syntax Batch #4.
- `tests/ok/implicit_gen.cp`: [E0201]: `cpl run` doesn't know the name `D` here — gap: see the error.
- `tests/ok/subset2.cp`: [E0309]: `|` can't be applied to `Set<V>` and `Set<V>` — gap: Syntax Batch #4.
- `tests/redteam/empty_slice_reduce.cp`: [E0302]: `if let` takes the value out of an optional, but this is a `Int` — gap: see the error.
- `tests/redteam/hashmap_sentinel.cp`: [E0305]: a `Counter_0` can't be indexed — gap: see the error.
- `tests/redteam/list_expr_once.cp`: [E0302]: a `Compressed<D_0, Int>` can't be taken apart into 2 names — gap: `compress` returns a record that a tuple pattern cannot split.

## Rejected: one-problem helpers

These came up in one port each and are already expressible with a general
mechanism. Building them would grow the prelude without helping a second
problem, which the minimal-core rule forbids. Revisit one only if a
second, unrelated port asks for the same thing.

- `sign(x)` (G20): `(x > 0) - (x < 0)` or a `match` on `cmp`.
- `mean` (L25), `board.with(p: v, q: w)` (L24), `(r, c).block(3, 3)` (L12).
- `g.bits(n)` (batch 25), `Side.flip` (batch 16), `Dsu.unite_under` (TR12).
- `Rooted.at_centroid` (TR6) as a constructor: centroid decomposition is a
  library algorithm over `Rooted`, not a new way to build one.
- A MinPlus-and-count semiring for Floyd-Warshall (MG12): a `Monoid` over a
  `(dist: ?Int, ways: Int)` record says it.
- `"MFB"[rand(0, 2)]` (`ioi07p4`) already works on a `Str`.

## Appendix: every `// WISH` marker by cluster

The text after the dash is the comment; the part in brackets is the backlog
heading its code points to.

### Judging: checkers, interactors and graders (17 markers, 13 ports)

- `scout_batch_26/ceoi22p3.cp`: TR16: an `interactor`/`ask` block would own the flush; the interpreter
- `scout_batch_23/ioi17p3.cp`: MG14: IOI grader-style function interface `pub fn who_wins(...)` vs stdin/stdout
- `scout_batch_40/tle18p5.cp`: `@grader const` for a constant the judge defines (see backlog C1). [C1: A judge-supplied compile-time constant has no `@grader` form (`tle18p5`)]
- `scout_batch_5/cco18p4.cp`: K21: the budget next to the question [K21: The question budget next to the question (2 of 10)]
- `scout_batch_5/yac6p3.cp`: K21 [K21: The question budget next to the question (2 of 10)]
- `scout_batch_13/btoi18p3.cp`: R10: a question asked twice is asked once [R10: Questions to a judge (2 of 10: btoi18p3, cpc21c1p5; batch 5 K7, K21)]
- `scout_batch_13/btoi18p3.cp`: K21 [K21: The question budget next to the question (2 of 10) [batch 5]]
- `scout_batch_13/btoi18p3.cp`: R10 [R10: Questions to a judge (2 of 10: btoi18p3, cpc21c1p5; batch 5 K7, K21)]
- `scout_batch_33/btoi18p2.cp`: C1: `count(need[s] > 0 for s in Sym)` is on the cheat sheet, not in the prelude. [C1: A judge-supplied compile-time constant has no `@grader` form (`tle18p5`) [batch 40]]
- `scout_batch_33/cco20p4.cp`: C1: `if all(seen): walk else: none`; `all`, `any`, `count` are not in the prelude. [C1: A judge-supplied compile-time constant has no `@grader` form (`tle18p5`) [batch 40]]
- `scout_batch_19/ioi16p5.cp`: B19-4: grader ABI (`extern fn` + `export fn`) unified with local `interactor:` [B19-4: Two-phase IOI grader APIs need `extern fn` / `export fn` with per-method limits and phase ordering]
- `scout_batch_19/ioi16p5.cp`: B19-4 + B19-3: stateful two-phase interactor with separate phase limits and power-of-2 generator [B19-4: Two-phase IOI grader APIs need `extern fn` / `export fn` with per-method limits and phase ordering; B19-3: Non-typeable, sparse, grammatical, and cross-test constraints need constructive `gen` + parameter shrinking]
- `scout_batch_19/ioi14p3.cp`: B19-5: Stateful exported module / class for reversed-control callback graders. [B19-5: Reversed-control adaptive graders need `export grader` and driver-mode `interactor: run(sol)`]
- `scout_batch_19/ioi14p3.cp`: B19-5: Driver-mode `interactor:` — drives the exported callbacks in an arbitrary (or adaptive!) [B19-5: Reversed-control adaptive graders need `export grader` and driver-mode `interactor: run(sol)`]
- `scout_batch_19/dmopc21c5p4.cp`: B19-7: per-case checker on multi-test output, validating permutation + prime differences + -1 [B19-7: Co-located `checker:` needs `read_output()` for variable-size outputs, multi-test loops, and impossibility cross-checks]
- `scout_batch_19/ccc23s3.cp`: B19-7: co-located `checker:` reusing input bindings (`n, m, r, c`) and typed output reading, [B19-7: Co-located `checker:` needs `read_output()` for variable-size outputs, multi-test loops, and impossibility cross-checks]
- `scout_batch_19/coci21c6p3.cp`: B19-7: variable-length multi-answer constructive `checker:` simulating every op forward [B19-7: Co-located `checker:` needs `read_output()` for variable-size outputs, multi-test loops, and impossibility cross-checks]

### Typed inputs for `cpl stress`: constraints a type cannot say (36 markers, 33 ports)

- `scout_batch_18/ccc22s5.cp`: W18: `Tree<V>` typed input generator is needed so `cpl stress` generates
- `scout_batch_18/ioi05p2.cp`: W7: sentinel-terminated heterogeneous command stream (`while tag != 'E'`) [W7: A `@brute` function with no parameters is run inside `main` (bug; draft `dpz`, `ccc19s4`) [batch 20]]
- `scout_batch_18/ioi05p2.cp`: W8: `pts.dedup()` in stdlib for coordinate compression [W8: Inputs that are only valid as the image of a hidden witness (`cco23p1`, `noi02p4`, `btoi17p5`, `ioi19p1`, `ioi09p8`) [batch 20]]
- `scout_batch_18/noi18p1.cp`: W21: `Graph(connected: true)` typed input constraint + edge record fields
- `scout_batch_18/noi18p1.cp`: W22: online encrypted queries depend on `lastans`, so `cpl stress` cannot
- `scout_batch_18/btoi17p2.cp`: W5: non-typeable layered-DAG edge constraint (b / k == a / k + 1) and no-duplicate-edge rule [W5: is a bug and is listed under (c). [batch 20]]
- `scout_batch_18/noi10p2.cp`: W10: non-typeable combinatorial upper bound on k: [W10: `Bitset` shifts one way only (`btoi19p2`) [batch 20]]
- `scout_batch_18/cco21p1.cp`: W4: HashMap<(Val, Val), Int> with unordered/ordered pair key without manual packing [W4: One input bounded by another (`ccc19s4`, `cco23p1`; draft `apio07p2`) [batch 20]]
- `scout_batch_18/noi09p4.cp`: W15: variable-length per-cell protection list cannot be read as one typed [W15: The loop-size warning fires on the `@brute` reference [batch 20]]
- `scout_batch_18/btoi20p3.cp`: W13: `scoped(dsu)` / `defer dsu.rollback(snap0)` would prevent [W13: Recursion, mutable state and `@brute` (draft `dpz`) [batch 20]]
- `scout_batch_3/aac4p3.cp`: G10: a reduced fraction; G5: Set from a generator [G10: A reduced fraction (aac4p3); G5: A set or a list straight from a generator (2 of 10: aac4p3, cco09p4)]
- `scout_batch_3/cco09p4.cp`: G5: a list of a whole domain [G5: A set or a list straight from a generator (2 of 10: aac4p3, cco09p4)]
- `scout_batch_23/dpi.cp`: MG11: floating-point range type `0.0..=1.0` for probability input & cpl stress
- `scout_batch_23/ioi17p3.cp`: MG11: graph constraint — every station must have out-degree >= 1
- `scout_batch_23/noi09p5.cp`: MG11: exact alphabet {'A', 'B'} and length constraints for cpl stress
- `scout_batch_23/noi07p1.cp`: MG11: simple undirected connected weighted graph constraint for cpl stress
- `scout_batch_23/egoi22p2.cp`: MG11: product constraint `w * h <= 250000` is non-typeable as independent ranges
- `scout_batch_23/ioi08p3.cp`: MG11: surjection constraint — every gem type 1..=k appears on at least one fish
- `scout_batch_23/coci21c1p2.cp`: MG11: alphabet constraint {'C', 'P'} + existential count >= 2*k - 1 for cpl stress
- `scout_batch_23/noi18p2.cp`: MG11: permutation input constraint `Perm<Val>` for cpl stress (not just `[Pos]Val`)
- `scout_batch_24/noi20p2.cp`: A9: Tree edge + strict-ancestor constraint `(u, v)` is non-typeable for
- `scout_batch_24/ceoi22p1.cp`: A21: Non-typeable input constraints: `n` must be even (`n % 2 == 0`) and
- `scout_batch_24/noip22p4.cp`: A6: Permutation constraint (`a` and `b` are permutations of 1..=n) cannot
- `scout_batch_5/cco22p1.cp`: K19: an ordered pair, read and generated as one [K19: Inputs with a relation between fields (cco22p1, and btoi17p1 through K4)]
- `scout_batch_5/btoi17p1.cp`: G5 [G5: A set or a list straight from a generator (2 of 10: aac4p3, cco09p4) [batch 3]]
- `scout_batch_5/btoi17p1.cp`: G5 [G5: A set or a list straight from a generator (2 of 10: aac4p3, cco09p4) [batch 3]]
- `scout_batch_13/coci21c4p4.cp`: R5: read and generated as a tree [R5: Inputs that come with a promise (5 of 10; extends batch 6 S4)]
- `scout_batch_13/ioi11p3.cp`: R5 [R5: Inputs that come with a promise (5 of 10; extends batch 6 S4)]
- `scout_batch_13/apio07p2.cp`: R5 [R5: Inputs that come with a promise (5 of 10; extends batch 6 S4)]
- `scout_batch_6/etsr23r1p2.cp`: S8, S4 [S8: Two numbers on one line, a mark between two numbers, and an order with a kind (6 of 10; batch 7 D11); S4: Bounds inside a record, and an input that is sorted (5 of 10; batch 8 H1)]
- `scout_batch_6/joi16fp2.cp`: S4: no way to say "sorted" to the generator [S4: Bounds inside a record, and an input that is sorted (5 of 10; batch 8 H1)]
- `scout_batch_6/cco17p5.cp`: S4 (H1) [S4: Bounds inside a record, and an input that is sorted (5 of 10; batch 8 H1); H1: A bound that names an earlier input is ignored (fixed in Stage-2 sprint #2; coci25c3p2) [batch 8]]
- `scout_batch_19/dmopc21c5p4.cp`: B19-3: multi-test sum-budget constraint `sum(ns) <= 1e6` expressed directly on the input vector [B19-3: Non-typeable, sparse, grammatical, and cross-test constraints need constructive `gen` + parameter shrinking]
- `scout_batch_19/noimps22p1.cp`: B19-3: stateful constructive `gen` for valid preprocessor programs (never `#undef`s an [B19-3: Non-typeable, sparse, grammatical, and cross-test constraints need constructive `gen` + parameter shrinking]
- `scout_batch_19/ceoi22p2.cp`: B19-3: constructive grammar `gen` (rejection-sampling random strings has 0% hit rate; [B19-3: Non-typeable, sparse, grammatical, and cross-test constraints need constructive `gen` + parameter shrinking]
- `scout_batch_19/ccc04s5.cp`: B19-3: constructive `gen` for sentinel-terminated multi-test streams guaranteeing at least [B19-3: Non-typeable, sparse, grammatical, and cross-test constraints need constructive `gen` + parameter shrinking]

### `read()` formats (26 markers, 23 ports)

- `scout_batch_18/dpw.cp`: W1: cross-field ordering `where l <= r` inside a record read array [W1: Plus Minus: the port is right where the accepted original is wrong [batch 20]]
- `scout_batch_3/dpr.cp`: G14: read straight into Mod [G14: Reading into `Mod` (dpr)]
- `scout_batch_3/modinv.cp`: G9: inputs past Int [G9: Numbers past `Int` (2 markers in modinv; decision 1.7 is leaning)]
- `scout_batch_3/coci06c1p6.cp`: G8: packed rows of a Char domain [G8: A grid of a Char domain is read and generated as numbers (bug; coci06c1p6)]
- `scout_batch_3/noip99p2.cp`: G3: the value of a digit character [G3: The value of a digit character (noip99p2)]
- `scout_batch_3/cco09p4.cp`: G22: an Order read from -1, 0, 1 [G22: Comparison results, and a three-way split (2 markers in cco09p4)]
- `scout_batch_26/ioi09p7.cp`: TR3: `let boss: [V]?V = read()` with "the root has no entry" said [TR3: trees given as a parent list]
- `scout_batch_26/ioi18p3.cp`: TR12: `Dsu<V>.unite_under(a, b)` (b becomes the top) or a ready [TR12: a tree built by union-find]
- `scout_batch_26/ceoi22p3.cp`: TR3: `let boss: [V]?V(none: -1) = read()` and `Rooted<V>.from_parents(boss)` [TR3: trees given as a parent list]
- `scout_batch_26/noip19p2.cp`: TR9: `let s: [V]Char = read()`; the interpreter reads a `[V]Char` as n
- `scout_batch_26/noip19p2.cp`: TR3: `let boss: [V]?V = read()` + `Rooted<V>.from_parents(boss)` [TR3: trees given as a parent list]
- `scout_batch_23/noimps22p4.cp`: MG11: input elements are constrained to be prime numbers <= 2000
- `scout_batch_12/noi05p1.cp`: L2: way: Dir4, numbered N S W E [L2: A direction that comes from the input (3 of 10)]
- `scout_batch_12/cco19p1.cp`: L20: a ?Player grid read from letters [L20: A cell that may be empty, read from a 0 or a dot (2 of 10)]
- `scout_batch_12/cco02p3.cp`: L2: var face: Dir4 = read(); face.cw, face.ccw [L2: A direction that comes from the input (3 of 10)]
- `scout_batch_12/noip09p4.cp`: L20: [Line, Line]?Digit, read with 0 for none [L20: A cell that may be empty, read from a 0 or a dot (2 of 10)]
- `scout_batch_12/ccc18s3.cp`: L2: Dir4.from("UDLR", ch) [L2: A direction that comes from the input (3 of 10)]
- `scout_batch_14/noip00p4.cp`: P3 [P3: Top-level `read()` inside a `while` loop silently swallows the rest of `main()` into the loop (bug; extends `S1`; `noip00p4` — 1 of 10; 44/710 corpus C++ files use sentinel/EOF `while` input loops)]
- `scout_batch_24/noi07p4.cp`: A4: Heterogeneous command tokens (`R`, `F`, `S`, `P`, `C`, `CS`) with
- `scout_batch_5/cco23p6.cp`: K13: the monoid lives with its record [K13: A monoid that is a record, and tables derived from the input (cco23p6; batch 1 F7)]
- `scout_batch_5/cco23p6.cp`: K13: a derived table that follows `have` [K13: A monoid that is a record, and tables derived from the input (cco23p6; batch 1 F7)]
- `scout_batch_13/tle17c5p6.cp`: K11: a wire is an input or a gate [K11: Vertices of several kinds (noi06p4) [batch 5]]
- `scout_batch_19/noimps22p1.cp`: B19-9: `read_line()` reads verbatim up to '\n' (preserving leading/trailing spaces and [B19-9: Verbatim line-oriented I/O (`read_line()`), sentinel-terminated streams (`read_until`), and byte-buffer `Str`]
- `scout_batch_19/ccc23s3.cp`: B19-6: optional output sentinel `none: "IMPOSSIBLE"` symmetric with `read(none: ...)` [B19-6: Symmetric `none:` sentinels on `read`/`print` and char-range `@tags` for mixed cell alphabets]
- `scout_batch_19/ccc04s5.cp`: B19-6: tagged char-cell type parsing '.', '*', and digits '1'..'9' cleanly without manual ASCII math [B19-6: Symmetric `none:` sentinels on `read`/`print` and char-range `@tags` for mixed cell alphabets]
- `scout_batch_19/ccc04s5.cp`: B19-9: sentinel-terminated multi-test loop (`read_until((0, 0))`) instead of `while true` + `break` [B19-9: Verbatim line-oriented I/O (`read_line()`), sentinel-terminated streams (`read_until`), and byte-buffer `Str`]

### Output formatting (30 markers, 20 ports)

- `scout_batch_3/noip01p4.cp`: G4 [G4: Text next to a value, and decimals (3 of 10: noip99p2, noip01p4, cco04p4)]
- `scout_batch_3/cco04p4.cp`: F15, G4 [F15: Floats (1 of 10; decision 1.5 is open) [batch 1]; G4: Text next to a value, and decimals (3 of 10: noip99p2, noip01p4, cco04p4)]
- `scout_batch_3/noip99p2.cp`: G4: text and a value with no space [G4: Text next to a value, and decimals (3 of 10: noip99p2, noip01p4, cco04p4)]
- `scout_batch_26/ceoi22p3.cp`: TR15: `print(picked)` should write the list on one line; the interpreter
- `scout_batch_23/dpi.cp`: MG9: fixed-precision float format `{ans:.10f}` (silently dropped by proto/cpi)
- `scout_batch_23/noi07p1.cp`: MG9: exact Int ratio -> F64 + `{total:.3f}` formatting
- `scout_batch_12/cco19p1.cp`: G4: decimals [G4: Text next to a value, and decimals (3 of 10: noip99p2, noip01p4, cco04p4) [batch 3]]
- `scout_batch_14/ccc20s5.cp`: P4 [P4: `Float` type, floating-point `@memo`, and precision-formatted `print` (extends `B5`; `noi05p4`, `ccc20s5` — 2 of 10; 53/710 corpus C++ files use floats, 30/710 format with `setprecision`/`%.Nf`)]
- `scout_batch_14/ccc20s5.cp`: P4 [P4: `Float` type, floating-point `@memo`, and precision-formatted `print` (extends `B5`; `noi05p4`, `ccc20s5` — 2 of 10; 53/710 corpus C++ files use floats, 30/710 format with `setprecision`/`%.Nf`)]
- `scout_batch_14/ccc20s5.cp`: P4 [P4: `Float` type, floating-point `@memo`, and precision-formatted `print` (extends `B5`; `noi05p4`, `ccc20s5` — 2 of 10; 53/710 corpus C++ files use floats, 30/710 format with `setprecision`/`%.Nf`)]
- `scout_batch_14/ccc20s5.cp`: P4 [P4: `Float` type, floating-point `@memo`, and precision-formatted `print` (extends `B5`; `noi05p4`, `ccc20s5` — 2 of 10; 53/710 corpus C++ files use floats, 30/710 format with `setprecision`/`%.Nf`)]
- `scout_batch_14/ccc20s5.cp`: P4 [P4: `Float` type, floating-point `@memo`, and precision-formatted `print` (extends `B5`; `noi05p4`, `ccc20s5` — 2 of 10; 53/710 corpus C++ files use floats, 30/710 format with `setprecision`/`%.Nf`)]
- `scout_batch_14/noi05p4.cp`: P4 [P4: `Float` type, floating-point `@memo`, and precision-formatted `print` (extends `B5`; `noi05p4`, `ccc20s5` — 2 of 10; 53/710 corpus C++ files use floats, 30/710 format with `setprecision`/`%.Nf`)]
- `scout_batch_14/noi05p4.cp`: P4 [P4: `Float` type, floating-point `@memo`, and precision-formatted `print` (extends `B5`; `noi05p4`, `ccc20s5` — 2 of 10; 53/710 corpus C++ files use floats, 30/710 format with `setprecision`/`%.Nf`)]
- `scout_batch_14/noi05p4.cp`: P4 [P4: `Float` type, floating-point `@memo`, and precision-formatted `print` (extends `B5`; `noi05p4`, `ccc20s5` — 2 of 10; 53/710 corpus C++ files use floats, 30/710 format with `setprecision`/`%.Nf`)]
- `scout_batch_14/noi05p4.cp`: P4 [P4: `Float` type, floating-point `@memo`, and precision-formatted `print` (extends `B5`; `noi05p4`, `ccc20s5` — 2 of 10; 53/710 corpus C++ files use floats, 30/710 format with `setprecision`/`%.Nf`)]
- `scout_batch_5/cco18p4.cp`: K5: today this allocates rows * cols slots [K5: `@memo` over two large domains allocates the whole table (fixed in Stage-2 sprint #2; cco18p4)]
- `scout_batch_5/yac6p3.cp`: G4-like: a record prints its fields [G4: Text next to a value, and decimals (3 of 10: noip99p2, noip01p4, cco04p4) [batch 3]]
- `scout_batch_33/cco05p6.cp`: C3: `stack.remove_at(i)`; `rotate_left` on a slice would also do [C3: Printing and rotating lists]
- `scout_batch_33/btoi15p2.cp`: C2: `print(if let t = below(i, 1): ops[t] else: 0)`; `if let` is a statement only. [C2: `Str` of unstated length against `Int` positions (`cheerio1s5`) [batch 40]]
- `scout_batch_33/cco20p4.cp`: C3: `walk.rotate_to(walk.find(c)!)`, then `print(walk)`; today the [C3: Printing and rotating lists]
- `scout_batch_6/nccc8s3.cp`: S8 [S8: Two numbers on one line, a mark between two numbers, and an order with a kind (6 of 10; batch 7 D11)]
- `scout_batch_6/cco04p2.cp`: S8 [S8: Two numbers on one line, a mark between two numbers, and an order with a kind (6 of 10; batch 7 D11)]
- `scout_batch_6/cco04p2.cp`: G4 [G4: Text next to a value, and decimals (3 of 10: noip99p2, noip01p4, cco04p4) [batch 3]]
- `scout_batch_6/etsr23r1p2.cp`: S8 [S8: Two numbers on one line, a mark between two numbers, and an order with a kind (6 of 10; batch 7 D11)]
- `scout_batch_6/etsr23r1p2.cp`: S8 [S8: Two numbers on one line, a mark between two numbers, and an order with a kind (6 of 10; batch 7 D11)]
- `scout_batch_6/joi16fp2.cp`: S8 [S8: Two numbers on one line, a mark between two numbers, and an order with a kind (6 of 10; batch 7 D11)]
- `scout_batch_6/joi16fp2.cp`: S8 [S8: Two numbers on one line, a mark between two numbers, and an order with a kind (6 of 10; batch 7 D11)]
- `scout_batch_6/dmopc21c8p1.cp`: S8 [S8: Two numbers on one line, a mark between two numbers, and an order with a kind (6 of 10; batch 7 D11)]
- `scout_batch_6/dmopc20c4p3.cp`: S8 [S8: Two numbers on one line, a mark between two numbers, and an order with a kind (6 of 10; batch 7 D11)]

### Grids, directions and 2-D views (37 markers, 18 ports)

- `scout_batch_32/noimps23p1.cp`: X6: the leftward pass is the same sweep on mirrored coordinates; the [X6: mirroring a 1D problem]
- `scout_batch_3/coci06c1p6.cp`: G7: hash of any rectangle [G7: A hash of any rectangle (coci06c1p6)]
- `scout_batch_3/coci06c1p6.cp`: G2: the mirror image of a range [G2: The mirror image, as a value (3 markers in 2 of 10: noip99p2, coci06c1p6)]
- `scout_batch_3/noip99p2.cp`: G2: a.rev as a value [G2: The mirror image, as a value (3 markers in 2 of 10: noip99p2, coci06c1p6)]
- `scout_batch_3/noip99p2.cp`: G2 [G2: The mirror image, as a value (3 markers in 2 of 10: noip99p2, coci06c1p6)]
- `scout_batch_12/noi05p1.cp`: L3: for line in Cell.lines(way) [L3: Lines of sight, and whole rows walked in a direction (2 of 10)]
- `scout_batch_12/coci10c7p4.cp`: L5: `for p in Cell` [L5: A pair of positions is not a position (7 of 10)]
- `scout_batch_12/coci10c7p4.cp`: L1: p.around() over Dir8 [L1: Moving on a grid (6 of 10)]
- `scout_batch_12/wossoly21s3.cp`: L1: p.step(d) over Dir4 [L1: Moving on a grid (6 of 10)]
- `scout_batch_12/wossoly21s3.cp`: L6: school.find('a') [L6: Where is the `S` (3 of 10)]
- `scout_batch_12/wossoly21s3.cp`: L7: bfs(from, p => (land(q) for q in p.around()))[to] [L7: A search from many sources, and one with free moves (4 of 10)]
- `scout_batch_12/cco19p1.cp`: L24: board.with(p: v, q: w), a copy with two cells changed
- `scout_batch_12/cco19p1.cp`: L1: p.around() over Dir4 [L1: Moving on a grid (6 of 10)]
- `scout_batch_12/cco02p3.cp`: L1: p.step(d) over Dir4 [L1: Moving on a grid (6 of 10)]
- `scout_batch_12/cco02p3.cp`: L14: with forward(): explore() steps back by itself [L14: A step that is undone on the way back (2 of 10)]
- `scout_batch_12/noip09p4.cp`: L12: (r, c).block(3, 3) [L12: The block a cell is in (noip09p4)]
- `scout_batch_12/noip09p4.cp`: L14: with mark(r, c, d): fill(...) undoes itself [L14: A step that is undone on the way back (2 of 10)]
- `scout_batch_12/ccc18s3.cp`: L1: p.step(d) over Dir4 [L1: Moving on a grid (6 of 10)]
- `scout_batch_12/ccc18s3.cp`: L5: `for cam in Cell` over a product domain [L5: A pair of positions is not a position (7 of 10)]
- `scout_batch_12/ccc18s3.cp`: L3: for q in cam.ray(d).take_while(q => g[q] != 'W') [L3: Lines of sight, and whole rows walked in a direction (2 of 10)]
- `scout_batch_12/ccc18s3.cp`: L6: let start = g.find('S')! [L6: Where is the `S` (3 of 10)]
- `scout_batch_12/dmopc21c6p3.cp`: L1: p.step(d) over Dir4 [L1: Moving on a grid (6 of 10)]
- `scout_batch_12/ioi05p1.cp`: G2: garden.flipped [G2: The mirror image, as a value (3 markers in 2 of 10: noip99p2, coci06c1p6) [batch 3]]
- `scout_batch_12/ioi05p1.cp`: L22: count.transposed [L22: A grid seen mirrored or turned (ioi05p1, 2 lines; batch 3 G2, batch 8 H15)]
- `scout_batch_24/cco18p5.cp`: A15: Clamping a window `[p - k + 1 .. p - 1]` around a domain index `p: I`
- `scout_batch_5/etsr23r3p2.cp`: F1, K17 [F1: Splitting on two optionals at once (`dmopc21c8p2`) [batch 44]; K17: Points with real coordinates, and a mirror image (etsr23r3p2; batch 1 F15, batch 3 G15)]
- `scout_batch_5/etsr23r3p2.cp`: F15, K17: points over a float [F15: Floats (1 of 10; decision 1.5 is open) [batch 1]; K17: Points with real coordinates, and a mirror image (etsr23r3p2; batch 1 F15, batch 3 G15)]
- `scout_batch_5/etsr23r3p2.cp`: K17 [K17: Points with real coordinates, and a mirror image (etsr23r3p2; batch 1 F15, batch 3 G15)]
- `scout_batch_5/cco22p1.cp`: G21, K14 [G21: `zip`, `cycle`, `take` (2 markers: cco04p4; noip99p2 uses `zip`) [batch 3]; K14: Windows of a grid and their sliding minimum (2 of 10; batch 1 F6)]
- `scout_batch_5/ioi06p2.cp`: F6, K14 [F6: Divisors from a sieve (`dmopc21c4p2`) [batch 44]; K14: Windows of a grid and their sliding minimum (2 of 10; batch 1 F6)]
- `scout_batch_5/ioi06p2.cp`: K14 [K14: Windows of a grid and their sliding minimum (2 of 10; batch 1 F6)]
- `scout_batch_5/ioi06p2.cp`: K14 [K14: Windows of a grid and their sliding minimum (2 of 10; batch 1 F6)]
- `scout_batch_13/btoi18p3.cp`: L1 [L1: Moving on a grid (6 of 10) [batch 12]]
- `scout_batch_13/btoi18p3.cp`: R11: a rectangle is a range of a product domain [R11: A rectangle, cut along its longer side (btoi18p3; batch 5 K14, batch 12 L5)]
- `scout_batch_13/btoi18p3.cp`: R11 [R11: A rectangle, cut along its longer side (btoi18p3; batch 5 K14, batch 12 L5)]
- `scout_batch_13/btoi18p3.cp`: R11: the cells beside c in `low` and in `high`, if any [R11: A rectangle, cut along its longer side (btoi18p3; batch 5 K14, batch 12 L5)]
- `scout_batch_13/ioi11p3.cp`: R3 [R3: A window whose test is the window (ioi11p3; extends batch 12 L8)]

### The search family (21 markers, 20 ports)

- `scout_batch_32/iati21a22.cp`: X3: "replace the first element >= v, else append" is LIS's whole [X3: LIS is still written by hand]
- `scout_batch_32/ioi04p2hard.cp`: X2: `Set` has `first_gt`, the ordered `Map` only `first_ge`. [X2: ordered Map has fewer lookups than Set]
- `scout_batch_32/996warmup8p3.cp`: X7: the search needs the simulated state at the answer, so the [X7: binary search whose predicate computes the answer state]
- `scout_batch_18/cco21p1.cp`: W3: stdlib binary-search helper `lower_bound(ys, p)` on sorted List [W3: "Strictly increasing" (dropped drafts `dpz`, `apio07p2`; extends `S4`) [batch 20]]
- `scout_batch_26/ioi09p7.cp`: TR4: `bsearch_first(0..w.len, i => w[i] >= p)` is on the cheat sheet
- `scout_batch_26/mmcc14p3.cp`: TR4: `bsearch_first(0..n, r => outposts(r) <= k)`
- `scout_batch_12/coci10c7p4.cp`: L7: let seen = reach(adj, from: town.where('P')) [L7: A search from many sources, and one with free moves (4 of 10)]
- `scout_batch_12/ccc18s3.cp`: L4: a Deque; WISH L7: shortest(moves, from: start) with weights 0..=1 [L4: No queue (3 of 10; batch 2 and batch 6 name `Deque`); L7: A search from many sources, and one with free moves (4 of 10)]
- `scout_batch_12/dmopc21c6p3.cp`: L7: bfs from many sources, in a given order [L7: A search from many sources, and one with free moves (4 of 10)]
- `scout_batch_5/noip10p3.cp`: K6 [K6: Searching a range of numbers or positions (3 of 10; batch 1 F10)]
- `scout_batch_5/ioi13p5.cp`: K6 [K6: Searching a range of numbers or positions (3 of 10; batch 1 F10)]
- `scout_batch_5/cco22p1.cp`: K6 [K6: Searching a range of numbers or positions (3 of 10; batch 1 F10)]
- `scout_batch_13/ceoi17p2.cp`: R9 [R9: A search over real numbers (ceoi17p2; extends batch 14 P4)]
- `scout_batch_13/coci21c4p4.cp`: R2 [R2: A search that keeps what the test found (coci21c4p4)]
- `scout_batch_13/ioi11p3.cp`: R1 [R1: The search family is half there (4 of 10; batch 5 K6, batch 7 D8)]
- `scout_batch_13/cpc21c1p5.cp`: R1 [R1: The search family is half there (4 of 10; batch 5 K6, batch 7 D8)]
- `scout_batch_13/coci12c1p4.cp`: R1 [R1: The search family is half there (4 of 10; batch 5 K6, batch 7 D8)]
- `scout_batch_13/tle17c5p6.cp`: R1 [R1: The search family is half there (4 of 10; batch 5 K6, batch 7 D8)]
- `scout_batch_6/cco04p2.cp`: S13 (D8), S6 [S13: The first key at or after a value (2 of 10, 1 dropped; batch 7 D8, batch 2 B1); D8: Binary search over a sorted list (sac22cc5p4, ccc14s4hard; batch 2 B1) [batch 7]; S6: `while let ... and`, and the last element as a place (4 of 10)]
- `scout_batch_6/etsr23r1p2.cp`: S13, S6 [S13: The first key at or after a value (2 of 10, 1 dropped; batch 7 D8, batch 2 B1); S6: `while let ... and`, and the last element as a place (4 of 10)]
- `scout_batch_6/etsr23r1p2.cp`: S13 [S13: The first key at or after a value (2 of 10, 1 dropped; batch 7 D8, batch 2 B1)]

### Optionals and patterns (21 markers, 21 ports)

- `scout_batch_32/ioi04p2hard.cp`: X1: `min(below.upto(p) + p, above.upto(-p) - p)` is the formula. [X1: arithmetic on an optional]
- `scout_batch_32/ceoi22p5.cp`: X1 again: shifting an optional by `cnt * 2 * d` needs a helper. [X1: arithmetic on an optional]
- `scout_batch_32/rpeng3.cp`: X1: this wants to be [X1: arithmetic on an optional]
- `scout_batch_18/ccc22s5.cp`: W19: fieldless enum as a domain/Index for `[V, State]?Int` DP table
- `scout_batch_18/noi09p4.cp`: W16: `stdlib/graph.cp`'s `topo` returns `none` on any cycle instead of [W16: Two compilers, one port (`cco17p4`, `btoi17p5`, `cco23p1`) [batch 20]]
- `scout_batch_17/apio07p2.cp`: if let (a, b) = (l, r)
- `scout_batch_3/cco04p4.cp`: G19: match as an expression [G19: `match` as an expression (cco04p4)]
- `scout_batch_26/ceoi22p3.cp`: TR17: `while let x = stack.pop():` (the cheat sheet's `pop() -> ?T`);
- `scout_batch_23/noi07p1.cp`: MG12: MinPlus + path-count semiring `(dist: ?Int, ways: Int)` for Floyd-Warshall
- `scout_batch_12/noip09p4.cp`: L13: let (r, c) = min_by(blanks(), p => options(p)) else: ... [L13: The best candidate, or none left (noip09p4; batch 5 K7 has `argmin`)]
- `scout_batch_12/ccc10s4.cp`: L19: mst(doors) -> ?(cost, edges), none when it cannot span [L19: A spanning tree (ccc10s4)]
- `scout_batch_24/ceoi22p1.cp`: A22: Representing "no next greater position" as `?Pos` is clean,
- `scout_batch_5/cco23p6.cp`: F1 [F1: Splitting on two optionals at once (`dmopc21c8p2`) [batch 44]]
- `scout_batch_5/noi06p4.cp`: F1, K12 [F1: Splitting on two optionals at once (`dmopc21c8p2`) [batch 44]; K12: Project selection as one call (noi06p4)]
- `scout_batch_5/noip10p3.cp`: F1, K8 [F1: Splitting on two optionals at once (`dmopc21c8p2`) [batch 44]; K8: A graph of some of the edges, and its 2-colouring (noip10p3; batch 1 F1)]
- `scout_batch_5/cco22p1.cp`: F1 [F1: Splitting on two optionals at once (`dmopc21c8p2`) [batch 44]]
- `scout_batch_13/coci21c4p4.cp`: F1 [F1: Splitting on two optionals at once (`dmopc21c8p2`) [batch 44]]
- `scout_batch_13/coci12c1p4.cp`: F1 [F1: Splitting on two optionals at once (`dmopc21c8p2`) [batch 44]]
- `scout_batch_6/nccc8s3.cp`: S6 [S6: `while let ... and`, and the last element as a place (4 of 10)]
- `scout_batch_6/dmopc20c4p3.cp`: S14 (F13, B1, D1) [S14: Counting in a sorted bag (1 of 10, 1 dropped; batch 1 F13, batch 2 B1, batch 7 D1); F13: A sorted bag with rank queries (1 of 10) [batch 1]; B1: Amounts are ranges, names are domains (4 of 10) [batch 4]; D1: No ordered bag, and none with ranks (noi04p1; batch 2 B1) [batch 7]]
- `scout_batch_6/cco17p5.cp`: S6 [S6: `while let ... and`, and the last element as a place (4 of 10)]

### Ranges, slices and sequence adapters (39 markers, 26 ports)

- `scout_batch_32/aac5p3.cp`: X5: intersecting two half-open ranges is spelled with max/min by hand. [X5: intersecting ranges]
- `scout_batch_18/ccc22s5.cp`: W20: `Rooted<V>.postorder()` helper (using `p in Rooted.Pre.rev => tree.at[p]` works but is non-obvious)
- `scout_batch_3/noip01p4.cp`: G16 [G16: All-pairs shortest paths (noip01p4)]
- `scout_batch_3/noip01p4.cp`: G16 [G16: All-pairs shortest paths (noip01p4)]
- `scout_batch_3/ccc18s4.cp`: G12: runs of equal w / k [G12: Runs of equal quotients (ccc18s4)]
- `scout_batch_3/cco04p4.cp`: F2: scan; G21: cycle [F2: Peeking at the last two elements of a stack (`dmopc21c9p2`) [batch 44]; G21: `zip`, `cycle`, `take` (2 markers: cco04p4; noip99p2 uses `zip`)]
- `scout_batch_3/cco04p4.cp`: G21, G15 [G21: `zip`, `cycle`, `take` (2 markers: cco04p4; noip99p2 uses `zip`); G15: Arithmetic on points in a port (2 of 10: noip01p4, cco04p4)]
- `scout_batch_3/aac4p3.cp`: G11 [G11: Pairs of equal keys (aac4p3)]
- `scout_batch_3/cco09p4.cp`: G22: a three-way split [G22: Comparison results, and a three-way split (2 markers in cco09p4)]
- `scout_batch_26/noi20p2.cp`: TR11: `var row = take(g[h])` to move the heavy child's row up
- `scout_batch_12/noi05p1.cp`: L16: for run in line.split(p => hall[p] == 'x') [L16: Stretches between obstacles (noi05p1)]
- `scout_batch_12/coci10c7p4.cp`: L8: for (lo, hi) in levels.stretches(delivers) [L8: For each left end, the first right end that is enough (2 of 10; batch 6 S12)]
- `scout_batch_12/cco19p1.cp`: G21: take; WISH L25: mean [G21: `zip`, `cycle`, `take` (2 markers: cco04p4; noip99p2 uses `zip`) [batch 3]]
- `scout_batch_12/ioi05p1.cp`: L8: for w in C.stretches(w => sum(strip[w]) >= k) [L8: For each left end, the first right end that is enough (2 of 10; batch 6 S12)]
- `scout_batch_12/ccc10s4.cp`: L18: for (j, a, b) in corner.cyc.pairs() [L18: Consecutive pairs round a ring (ccc10s4; batch 1 F11)]
- `scout_batch_14/csps21p2.cp`: P6 [P6: Boundary-based open domain subranges (`l.r..r.l`) and same-domain distance `r - l` (`csps21p2`, `dmopc18c1p6` — 2 of 10; applies to almost all interval DP)]
- `scout_batch_14/csps21p2.cp`: P6 [P6: Boundary-based open domain subranges (`l.r..r.l`) and same-domain distance `r - l` (`csps21p2`, `dmopc18c1p6` — 2 of 10; applies to almost all interval DP)]
- `scout_batch_14/csps21p2.cp`: P6, P9 [P6: Boundary-based open domain subranges (`l.r..r.l`) and same-domain distance `r - l` (`csps21p2`, `dmopc18c1p6` — 2 of 10; applies to almost all interval DP); P9: `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)]
- `scout_batch_14/csps21p2.cp`: P6 [P6: Boundary-based open domain subranges (`l.r..r.l`) and same-domain distance `r - l` (`csps21p2`, `dmopc18c1p6` — 2 of 10; applies to almost all interval DP)]
- `scout_batch_14/csps21p2.cp`: P6, P9 [P6: Boundary-based open domain subranges (`l.r..r.l`) and same-domain distance `r - l` (`csps21p2`, `dmopc18c1p6` — 2 of 10; applies to almost all interval DP); P9: `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)]
- `scout_batch_14/csps21p2.cp`: P6 [P6: Boundary-based open domain subranges (`l.r..r.l`) and same-domain distance `r - l` (`csps21p2`, `dmopc18c1p6` — 2 of 10; applies to almost all interval DP)]
- `scout_batch_14/csps21p2.cp`: P6, P9 [P6: Boundary-based open domain subranges (`l.r..r.l`) and same-domain distance `r - l` (`csps21p2`, `dmopc18c1p6` — 2 of 10; applies to almost all interval DP); P9: `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)]
- `scout_batch_14/csps21p2.cp`: P6 [P6: Boundary-based open domain subranges (`l.r..r.l`) and same-domain distance `r - l` (`csps21p2`, `dmopc18c1p6` — 2 of 10; applies to almost all interval DP)]
- `scout_batch_14/noi15p3.cp`: P9 [P9: `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)]
- `scout_batch_14/noi15p3.cp`: P9 [P9: `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)]
- `scout_batch_14/noi15p3.cp`: P9 [P9: `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)]
- `scout_batch_14/noi15p3.cp`: P5, P9 [P5: Auto-derived `Index` for fieldless `enum`s and finite arrays `[D]E` (extends `B11`; `noi15p3` — 1 of 10; generalizes to all k-ary bitmask, plug/profile, and digit-DP state spaces); P9: `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)]
- `scout_batch_14/noip21p2.cp`: P9 [P9: `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)]
- `scout_batch_14/noip21p2.cp`: P11, P9 [P11: Bit-level intrinsics on `Int` (`popcount`, `ctz`, `clz`) and `Subset<D>.shifted(k)` (extends `H12`; `noip21p2`, `noi23p2` — 2 of 10; 22/710 corpus C++ files call `__builtin_popcount`, 145/710 shift bitmasks); P9: `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)]
- `scout_batch_14/dmopc18c1p6.cp`: P9 [P9: `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)]
- `scout_batch_14/noi05p4.cp`: P9 [P9: `for ... if cond:` loop filters and multiline `if / elif / else` expressions (`csps21p2`, `dmopc18c1p6`, `noip21p2`, `noi15p3`, `noi05p4` — 5 of 10)]
- `scout_batch_24/noi20p2.cp`: A11: `Rooted<V>` still lacks `.postorder()` / `.order.rev` iteration
- `scout_batch_5/cco23p6.cp`: F7 [F7: All-pairs BFS has no direct spelling (`dmopc21c9p3`) [batch 44]]
- `scout_batch_5/ioi13p5.cp`: K10 [K10: Sorting downwards (ioi13p5)]
- `scout_batch_13/ceoi17p2.cp`: K10 [K10: Sorting downwards (ioi13p5) [batch 5]]
- `scout_batch_13/ioi14p6.cp`: K10: .rev on a slice [K10: Sorting downwards (ioi13p5) [batch 5]]
- `scout_batch_33/ioi12p6.cp`: C7: an order-statistics `Set` (`line.nth(s)`, `line.remove_range(s..=e)`) [C7: No order-statistics container]
- `scout_batch_33/ccc20s4.cp`: C5: `let before = [Group] g => prefix(s.cyclic, c => c == g.letter)` [C5: Circular ranges]
- `scout_batch_6/coci11c4p4.cp`: S16, G21 [S16: Sorting positions, and handing out in order (1 of 10; batch 3 G21); G21: `zip`, `cycle`, `take` (2 markers: cco04p4; noip99p2 uses `zip`) [batch 3]]

### Numbers: Mod, wide ints, bits, polynomials, floats (34 markers, 23 ports)

- `scout_batch_18/btoi20p3.cp`: W12: bare-core fallback — `stdlib/dsu.cp` has path-halving DSU only, [W12: `choose` over `Mod` worked on the first try [batch 20]]
- `scout_batch_3/dpr.cp`: G13: binary power of anything [G13: A binary power of anything (dpr)]
- `scout_batch_3/modinv.cp`: G9: wide intermediates [G9: Numbers past `Int` (2 markers in modinv; decision 1.7 is leaning)]
- `scout_batch_3/noip01p4.cp`: F15 [F15: Floats (1 of 10; decision 1.5 is open) [batch 1]]
- `scout_batch_3/coci06c1p6.cp`: F6 [F6: Divisors from a sieve (`dmopc21c4p2`) [batch 44]]
- `scout_batch_3/cco04p4.cp`: G20: sign [G20: `sign` (cco04p4)]
- `scout_batch_23/dpi.cp`: MG6: `Poly<F64>.product([(1.0 - p[i], p[i]) for i in Coin])` (divide-and-conquer
- `scout_batch_23/noimps22p4.cp`: MG7: SOS / Zeta-Mobius subset transform is missing from std.poly / std.bits
- `scout_batch_23/noimps22p4.cp`: MG13: passing a lent 2D row `mut big_sub[b]` into `sos(mut a: [Mask]Int)`
- `scout_batch_23/noimps22p4.cp`: MG8: submask iteration `for sub in submasks(sm)` instead of manual `while`
- `scout_batch_23/noimps22p4.cp`: MG4: integer/modular promotion & sign parity `(-1)^popcount(sub)`
- `scout_batch_23/egoi22p2.cp`: MG6: Formal power series `Poly<Z>.inv(w + 1)` in std.poly (using NTT for
- `scout_batch_23/noi19p5.cp`: MG6: Poly<Z> with eval(x) and interpolate(pts) in std.poly
- `scout_batch_23/noi19p5.cp`: MG4: constant/Int -> Mod<P> promotion would eliminate Z(k), Z(r), Z(k - 1), Z(r - 1)
- `scout_batch_23/ioi08p3.cp`: MG3: Monoid<Int> capturing runtime `mod` collides with static monoid specialization!
- `scout_batch_12/cco19p1.cp`: L23: @memo keyed by an array value [L23: , L24, L25. A game on a whole board with a real answer (cco19p1, sketch)]
- `scout_batch_14/noip00p4.cp`: P10 [P10: Generic Zeta / Möbius (SOS) transforms over `Subset<D>` parameterized by `Algebra` (`noip00p4`, `noi15p3` — 2 of 10)]
- `scout_batch_14/noip00p4.cp`: P10 [P10: Generic Zeta / Möbius (SOS) transforms over `Subset<D>` parameterized by `Algebra` (`noip00p4`, `noi15p3` — 2 of 10)]
- `scout_batch_14/noip00p4.cp`: P10 [P10: Generic Zeta / Möbius (SOS) transforms over `Subset<D>` parameterized by `Algebra` (`noip00p4`, `noi15p3` — 2 of 10)]
- `scout_batch_14/noip00p4.cp`: P10 [P10: Generic Zeta / Möbius (SOS) transforms over `Subset<D>` parameterized by `Algebra` (`noip00p4`, `noi15p3` — 2 of 10)]
- `scout_batch_14/noi23p2.cp`: P11 [P11: Bit-level intrinsics on `Int` (`popcount`, `ctz`, `clz`) and `Subset<D>.shifted(k)` (extends `H12`; `noip21p2`, `noi23p2` — 2 of 10; 22/710 corpus C++ files call `__builtin_popcount`, 145/710 shift bitmasks)]
- `scout_batch_14/noi23p2.cp`: P11 [P11: Bit-level intrinsics on `Int` (`popcount`, `ctz`, `clz`) and `Subset<D>.shifted(k)` (extends `H12`; `noip21p2`, `noi23p2` — 2 of 10; 22/710 corpus C++ files call `__builtin_popcount`, 145/710 shift bitmasks)]
- `scout_batch_14/noi15p3.cp`: P7 [P7: Compiling `stdlib/mod.cp` (`ModR`, `Comb<P>`) directly instead of `proto/cpi`-only stand-ins (extends `B9`/`H1`; `noi15p3`, `noip21p2` — 2 of 10)]
- `scout_batch_14/noi15p3.cp`: P7 [P7: Compiling `stdlib/mod.cp` (`ModR`, `Comb<P>`) directly instead of `proto/cpi`-only stand-ins (extends `B9`/`H1`; `noi15p3`, `noip21p2` — 2 of 10)]
- `scout_batch_14/noip21p2.cp`: P7 [P7: Compiling `stdlib/mod.cp` (`ModR`, `Comb<P>`) directly instead of `proto/cpi`-only stand-ins (extends `B9`/`H1`; `noi15p3`, `noip21p2` — 2 of 10)]
- `scout_batch_14/noip21p2.cp`: P7 [P7: Compiling `stdlib/mod.cp` (`ModR`, `Comb<P>`) directly instead of `proto/cpi`-only stand-ins (extends `B9`/`H1`; `noi15p3`, `noip21p2` — 2 of 10)]
- `scout_batch_24/ioi13p3io.cp`: A17: Heavy value-type `Monoid<Mat>` collision with `SegTree<D, T>`!
- `scout_batch_24/ioi13p3io.cp`: A18: `Monoid<Mat>` requires an identity value (`Mat.identity()`),
- `scout_batch_25/noip19p1.cp`: `g.bits(n)`, most significant first
- `scout_batch_25/noi20p4.cp`: Q8: a bitset over a signed domain
- `scout_batch_1/ccc05s5.cp`: F15: floats are still undecided [F15: Floats (1 of 10; decision 1.5 is open)]
- `scout_batch_1/ccc19s5.cp`: F4: a triangular index [F4: A triangular index (1 of 10, 1 earlier port)]
- `scout_batch_6/coci11c4p4.cp`: S17, G20 [S17: Neighbours in a line (2 of 10); G20: `sign` (cco04p4) [batch 3]]
- `scout_batch_6/dmopc20c7p1.cp`: S11 (B5) [S11: A max-heap, and a heap of the largest count (2 of 10, 1 dropped; batch 2 B5); B5: Floats (1 problem dropped; batch 1 F15; decision 1.5 is open) [batch 4]]

### Trees (15 markers, 12 ports)

- `scout_batch_18/cco21p4.cp`: W14: `sorted(E, by: e => -roads[e].r)` allocates a List<E> and runs O(m log m) [W14: Typed positions against block, tree and coordinate arithmetic (`ccc19s4`, `ioi09p8`, draft `dpz`) [batch 20]]
- `scout_batch_18/ioi05p2.cp`: W9: `LazySegTree.max_right` is missing from `stdlib/lazy.cp` [W9: `use std.` is refused by `proto/cpc` (`cco17p4`, `noi02p4`, `ceoi18p5` — 3 of 10; extends `B9`, `H1`, `P7`) [batch 20]]
- `scout_batch_18/noi18p1.cp`: W23: Multi-feature collision — Kruskal reconstruction tree extends domain V
- `scout_batch_18/noi18p1.cp`: W24: `stdlib/tree.cp` (`Rooted`) does not expose a generic binary-lifting
- `scout_batch_26/noip19p6.cp`: TR6: `let t = Rooted<V>.at_centroid(g)` (or `t.centroid()` and [TR6: `reroot`]
- `scout_batch_14/noi19p1.cp`: P8, P12 [P8: In-file `struct` methods and `use local.` imports (`ioi16p6io`, `noi19p1` — 2 of 10; 255/710 corpus C++ files define a `struct`); P12: Generic `LiChao<F, V>` and `LiChaoArena<F, V>` for tie-breaking values and multi-tree DP (`ioi16p6io`, `noi19p1` — 2 of 10; 9/710 corpus C++ files use CHT/Li Chao)]
- `scout_batch_14/noi19p1.cp`: P12 [P12: Generic `LiChao<F, V>` and `LiChaoArena<F, V>` for tie-breaking values and multi-tree DP (`ioi16p6io`, `noi19p1` — 2 of 10; 9/710 corpus C++ files use CHT/Li Chao)]
- `scout_batch_14/ioi16p6io.cp`: P8, P12 [P8: In-file `struct` methods and `use local.` imports (`ioi16p6io`, `noi19p1` — 2 of 10; 255/710 corpus C++ files define a `struct`); P12: Generic `LiChao<F, V>` and `LiChaoArena<F, V>` for tie-breaking values and multi-tree DP (`ioi16p6io`, `noi19p1` — 2 of 10; 9/710 corpus C++ files use CHT/Li Chao)]
- `scout_batch_14/ioi16p6io.cp`: P12 [P12: Generic `LiChao<F, V>` and `LiChaoArena<F, V>` for tie-breaking values and multi-tree DP (`ioi16p6io`, `noi19p1` — 2 of 10; 9/710 corpus C++ files use CHT/Li Chao)]
- `scout_batch_24/ioi07p3.cp`: A2: `LazySegTree` lacks `max_right` and `min_left`, forcing either
- `scout_batch_24/cco16p5.cp`: A23: Cover-Count Segment Tree structurally breaks `LazySegTree<D, T, F>`!
- `scout_batch_24/noi20p2.cp`: A10: `Pool<Node>` + `Node.Id` implicit tree ergonomics:
- `scout_batch_24/noip22p4.cp`: A7: Heavy custom Monoid + Action boilerplate for historical-sum lazy trees.
- `scout_batch_24/cco18p5.cp`: A14: `stdlib/segtree.cp` has `Max<Int>`, which returns only the max value,
- `scout_batch_13/coci21c4p4.cp`: B14: keeps the road lengths [B14: A rooted tree forgets its edge data (1 of 10; batch 1 F16) [batch 4]]

### Graphs (3 markers, 3 ports)

- `scout_batch_18/noi09p4.cp`: W17: Multi-feature collision — `FlowGraph<Node>` cannot be parameterized
- `scout_batch_3/noip01p4.cp`: G17 [G17: `unreachable` (noip01p4)]
- `scout_batch_5/noip10p3.cp`: K8: a graph of some of the edges [K8: A graph of some of the edges, and its 2-colouring (noip10p3; batch 1 F1)]

### Range structures and monoids (13 markers, 13 ports)

- `scout_batch_18/noi10p2.cp`: W11: `SparseTable<I.bounds, ...>` over a boundary domain + argmax RMQ helper [W11: Small missing spellings in `proto/cpc` [batch 20]]
- `scout_batch_23/noi18p2.cp`: MG1: static let Comb across @multitest with a lazy or max-bound table
- `scout_batch_12/noi05p1.cp`: L15: sliding(max, xs, width) over a MonoQueue [L15: The largest of the last few (noi05p1; batch 5 K14)]
- `scout_batch_14/noip00p4.cp`: P14 [P14: `List.is_empty`, `List.dedup()`, and header-prefixed multitests (`noip00p4`, `ioi16p6io`, `noi23p2` — 3 of 10; 30/710 corpus C++ files call `unique()`)]
- `scout_batch_14/noi23p2.cp`: P14 [P14: `List.is_empty`, `List.dedup()`, and header-prefixed multitests (`noip00p4`, `ioi16p6io`, `noi23p2` — 3 of 10; 30/710 corpus C++ files call `unique()`)]
- `scout_batch_24/ioi07p3.cp`: A1: `MinOpt` and `AddMin` are missing from `stdlib/lazy.cp` even though
- `scout_batch_24/noi07p4.cp`: A5: `opt(Semigroup)` wraps `Run` in `?Run` cleanly for the identity, but
- `scout_batch_1/ccc05s5.cp`: F13: a bag with rank queries [F13: A sorted bag with rank queries (1 of 10)]
- `scout_batch_5/etsr23r3p2.cp`: F5-like: the fold left of a boundary [F5: "k largest" (`dmopc21c6p2`) [batch 44]]
- `scout_batch_13/ioi14p6.cp`: the k largest of a prefix, summed
- `scout_batch_13/apio07p2.cp`: R7, S17 [R7: Divide and conquer with an associative join is a fold (apio07p2); S17: Neighbours in a line (2 of 10) [batch 6]]
- `scout_batch_33/ioi09p5.cp`: C7: `var free = Set<Space>.full()` with `free.pop_first()`; the [C7: No order-statistics container]
- `scout_batch_6/dmopc20c4p3.cp`: S14 [S14: Counting in a sorted bag (1 of 10, 1 dropped; batch 1 F13, batch 2 B1, batch 7 D1)]

### State: memo tables, undo, pools, cursors (8 markers, 6 ports)

- `scout_batch_23/coci21c1p2.cp`: MG10: @memo over 3D bounded (l, r, my_red) with dense flat table + packed Bool/U8
- `scout_batch_14/noi15p3.cp`: P5 [P5: Auto-derived `Index` for fieldless `enum`s and finite arrays `[D]E` (extends `B11`; `noi15p3` — 1 of 10; generalizes to all k-ary bitmask, plug/profile, and digit-DP state spaces)]
- `scout_batch_14/noi15p3.cp`: P5 [P5: Auto-derived `Index` for fieldless `enum`s and finite arrays `[D]E` (extends `B11`; `noi15p3` — 1 of 10; generalizes to all k-ary bitmask, plug/profile, and digit-DP state spaces)]
- `scout_batch_14/noi15p3.cp`: P5 [P5: Auto-derived `Index` for fieldless `enum`s and finite arrays `[D]E` (extends `B11`; `noi15p3` — 1 of 10; generalizes to all k-ary bitmask, plug/profile, and digit-DP state spaces)]
- `scout_batch_24/noi05p2.cp`: A13: `Pool<Node>` with free-list recycling is mandatory at 64 MB when
- `scout_batch_24/dmpg15g6.cp`: A19: `Journaled<[Flav]Int>` + `undo` blocks (`plan/14-state.md`)
- `scout_batch_5/ioi13p5.cp`: K9: a cursor that survives the loop [K9: Heaps: biggest first, a look at the top, up to k at once (2 of 10; batch 2 B5)]
- `scout_batch_13/tle17c5p6.cp`: a memo that lives for one call of `light`

### Maps, sets, heaps and keys (13 markers, 9 ports)

- `scout_batch_32/ceoi22p5.cp`: X4: equal positions need distinct leaves, and `compress` merges equal [X4: rank compression with ties]
- `scout_batch_17/ioi07p4.cp`: "MFB"[rand(0, 2)]
- `scout_batch_3/cco09p4.cp`: G23: a random element [G23: A random element (cco09p4)]
- `scout_batch_3/cco09p4.cp`: G24 [G24: The single element (cco09p4)]
- `scout_batch_12/ccc10s4.cp`: L17: beside[wall].push(...) on a key that is new [L17: An unordered pair as a key, and a list under a new key (ccc10s4)]
- `scout_batch_24/noip22p4.cp`: A8: Offline query bucketing (`[I]List<Q>`) and answering into `[Q]U64`
- `scout_batch_24/dmpg15g6.cp`: A20: Offline Mo's query bucketing + sorting: partitioning short vs
- `scout_batch_5/ioi13p5.cp`: K9 [K9: Heaps: biggest first, a look at the top, up to k at once (2 of 10; batch 2 B5)]
- `scout_batch_5/ioi13p5.cp`: K9: pop up to k [K9: Heaps: biggest first, a look at the top, up to k at once (2 of 10; batch 2 B5)]
- `scout_batch_5/btoi17p1.cp`: K9: a heap that follows its key [K9: Heaps: biggest first, a look at the top, up to k at once (2 of 10; batch 2 B5)]
- `scout_batch_5/btoi17p1.cp`: B6 (batch 4): set algebra [B6: Subsets of a subset, and set algebra (1 of 10) [batch 4]]
- `scout_batch_5/btoi17p1.cp`: B6, K18 [B6: Subsets of a subset, and set algebra (1 of 10) [batch 4]; K18: Every subset of a small list (btoi17p1; batch 4 B6)]
- `scout_batch_33/cco21p2.cp`: C4: `HashMap<Int, Int>` is on the cheat sheet; the interpreter only has `Map`. [C4: `max_by` on an empty domain and on ties (`tle18p5`) [batch 40]]

### Records, enums and methods (18 markers, 14 ports)

- `scout_batch_32/aac5p3.cp`: X8: the cheat sheet only shows empty constructors (`SortedBag<T>()`); [X8: building ordered containers from a sequence]
- `scout_batch_3/modinv.cp`: F1 [F1: again. Ports cannot use the standard library (5 of 10)]
- `scout_batch_3/noip01p4.cp`: F1; dist is G15 [F1: again. Ports cannot use the standard library (5 of 10); G15: Arithmetic on points in a port (2 of 10: noip01p4, cco04p4)]
- `scout_batch_3/noip01p4.cp`: G18: a record of domains as an index [G18: A record of domains as an index (noip01p4)]
- `scout_batch_3/ccc03s4.cp`: F1 [F1: again. Ports cannot use the standard library (5 of 10)]
- `scout_batch_3/cco04p4.cp`: F1 [F1: again. Ports cannot use the standard library (5 of 10)]
- `scout_batch_3/cco04p4.cp`: G25: a record with an enum field [G25: A record with an enum field builds, and the C++ does not (bug; cco04p4)]
- `scout_batch_23/ioi08p3.cp`: MG2: ModR is an operation struct, not a value type with `+ - *` operators;
- `scout_batch_12/wossoly21s3.cp`: L9: ch.is_lower, ch.lower [L9: Kinds of letters (wossoly21s3; batch 3 G3)]
- `scout_batch_14/ioi16p6io.cp`: P8 [P8: In-file `struct` methods and `use local.` imports (`ioi16p6io`, `noi19p1` — 2 of 10; 255/710 corpus C++ files define a `struct`)]
- `scout_batch_14/ioi16p6io.cp`: P2 [P2: `var (a, b) = (x, y)` destructuring binds variables as immutable `let` (bug; `noip21p2`, `ioi16p6io`, `noi19p1` — 3 of 10)]
- `scout_batch_14/noi15p3.cp`: P2 [P2: `var (a, b) = (x, y)` destructuring binds variables as immutable `let` (bug; `noip21p2`, `ioi16p6io`, `noi19p1` — 3 of 10)]
- `scout_batch_14/noi15p3.cp`: P2 [P2: `var (a, b) = (x, y)` destructuring binds variables as immutable `let` (bug; `noip21p2`, `ioi16p6io`, `noi19p1` — 3 of 10)]
- `scout_batch_14/dmopc18c1p6.cp`: P13 [P13: `min_by` / `argmin` returning both the best value and winning key (`dmopc18c1p6`, `noi05p4` — 2 of 10; applies to Knuth DP, D&C DP, and DP path reconstruction)]
- `scout_batch_14/noi05p4.cp`: P13 [P13: `min_by` / `argmin` returning both the best value and winning key (`dmopc18c1p6`, `noi05p4` — 2 of 10; applies to Knuth DP, D&C DP, and DP path reconstruction)]
- `scout_batch_24/noi05p2.cp`: A12: Variable-length payload inside an enum variant (`INSERT posi tot c_1..c_tot`)
- `scout_batch_5/noi06p4.cp`: K11: an enum of domains as an index [K11: Vertices of several kinds (noi06p4)]
- `scout_batch_16/ioi07p2.cp`: Side.flip = the other side

### Long tail (single-problem wishes) (32 markers, 22 ports)

- `scout_batch_18/btoi17p2.cp`: W6: 2D array comprehension over product domain `[Slot, Slot] (i, j) => ...` [W6: Tuple assignment after `var (a, b) = ...` assigns to a temporary (bug; `noi02p4`; extends `P2`) [batch 20]]
- `scout_batch_18/cco21p1.cp`: W2: adjacency swap index is 1..=k-1, a boundary/gap domain over Val (`Val.gaps` or `1..k`) [W2: Six problems came out as a smaller algorithm, not a smaller transcription [batch 20]]
- `scout_batch_17/ioi13p1.cp`: max(walks, by: .dist).to
- `scout_batch_3/ccc03s4.cp`: G6: positions of s, not Ints [G6: A suffix array that knows what it sorts (ccc03s4)]
- `scout_batch_26/noi20p2.cp`: TR10: this row is a dense `List<Z>` of length depth(x) + 1, so a
- `scout_batch_12/noip09p4.cp`: L11: in_row[r].toggle(d), or in_row[r] ^= d [L11: Putting a member in and taking it out (noip09p4)]
- `scout_batch_14/ioi16p6io.cp`: P1 [P1: Emitted `using namespace std;` collides with ordinary user variable names (bug; `ioi16p6io` — 1 of 10; 34/710 corpus C++ files use `norm` alone)]
- `scout_batch_40/cheerio1s5.cp`: Int positions into a string of unstated length (see backlog C2). [C2: `Str` of unstated length against `Int` positions (`cheerio1s5`)]
- `scout_batch_24/ioi07p3.cp`: A3: advancing a `Bounds<H>` by an integer offset `rem` has no
- `scout_batch_5/cco23p6.cp`: K16 [K16: A number that may or may not be a position (2 of 10)]
- `scout_batch_5/cco18p4.cp`: K7 [K7: The minimum of a convex function (cco18p4)]
- `scout_batch_5/yac6p3.cp`: K16: a checked conversion [K16: A number that may or may not be a position (2 of 10)]
- `scout_batch_13/ceoi17p2.cp`: R8: two lists, two hand-moved places [R8: Convex cost lists, and two lists walked together (apio07p2, ceoi17p2)]
- `scout_batch_13/ioi14p6.cp`: R6 [R6: A best choice fenced in by its neighbours (ioi14p6; relates batch 14 P13)]
- `scout_batch_13/ioi14p6.cp`: R6 [R6: A best choice fenced in by its neighbours (ioi14p6; relates batch 14 P13)]
- `scout_batch_13/apio07p2.cp`: R8 [R8: Convex cost lists, and two lists walked together (apio07p2, ceoi17p2)]
- `scout_batch_13/ioi05p4.cp`: H6: indexed by a seat [H6: A permutation has no cycles, and is not indexed by its values (sac22cc4jp4) [batch 8]]
- `scout_batch_13/ioi05p4.cp`: S17 [S17: Neighbours in a line (2 of 10) [batch 6]]
- `scout_batch_13/ioi05p4.cp`: S17 [S17: Neighbours in a line (2 of 10) [batch 6]]
- `scout_batch_6/coci11c4p4.cp`: S17 [S17: Neighbours in a line (2 of 10)]
- `scout_batch_6/coci11c4p4.cp`: S10 [S10: A table that is filled later has no honest first value (2 of 10, 1 dropped)]
- `scout_batch_6/coci11c4p4.cp`: S17 [S17: Neighbours in a line (2 of 10)]
- `scout_batch_6/joi16fp2.cp`: S17 [S17: Neighbours in a line (2 of 10)]
- `scout_batch_6/joi16fp2.cp`: S19 [S19: The largest k, and descending order (4 of 10)]
- `scout_batch_6/dmopc21c8p1.cp`: S12 [S12: A sorted list that is drained from either end (2 of 10)]
- `scout_batch_6/dmopc21c8p1.cp`: S12 [S12: A sorted list that is drained from either end (2 of 10)]
- `scout_batch_6/dmopc21c8p1.cp`: S12 [S12: A sorted list that is drained from either end (2 of 10)]
- `scout_batch_6/dmopc21c8p1.cp`: S7 [S7: Leaving the program early (1 of 10, 1 dropped)]
- `scout_batch_6/dmopc20c4p3.cp`: S12 [S12: A sorted list that is drained from either end (2 of 10)]
- `scout_batch_6/dmopc20c4p3.cp`: S12 [S12: A sorted list that is drained from either end (2 of 10)]
- `scout_batch_6/cco17p5.cp`: S15 [S15: Grouping by a key (2 of 10, 1 dropped)]
- `scout_batch_6/ccc21s3.cp`: S9 [S9: A comprehension with two `for`s (1 of 10)]

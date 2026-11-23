# Friction Backlog — Scout Batch 49: Hard IOI Classics, Part 2

Ten unported problems from the `ioi*` bucket were rethought in CP Lang under `proto/ports/scout_batch_49/`: `ioi00p2` (Car Parking), `ioi01p3` (Twofive), `ioi99p6` (A Strip of Land), `ioi14p4` (Gondola), `ioi15p5` (Sorting), `ioi17p2` (Wiring), `ioi21p1` (Distributing Candies), `ioi22p4` (Digital Circuit), `ioi12p2` (Parachute Rings) and `ioi18p2` (Seats).

Each has a v2-spec file (`<problem>.cp`, written against `docs/one-page-cheat-sheet.md`, not compiled) and a buildable file (`today/<problem>.cp`, compiled through the unmodified `proto/cpc`). The buildable files agree with the original accepted C++ on 300 random inputs each (200 small, 100 medium, 3,000 total), with the port side under ASan and UBSan. Four of them also agree with an independent brute force, and four have their own output replayed by a validator. The limits of that check are in the batch README; the main ones are that the v2 files are verified by reading only, that the seven grader problems are compared through a stdin/stdout wrapper, and that nothing was run at maximum size.

All ten were written from the accepted C++ and the published IOI tasks, because the corpus carries no statements. Four ports use a different method from the original (`ioi17p2`, `ioi21p1`, `ioi12p2`, `ioi22p4`); the README says how.

## Dual LOC

Counted by `proto/ports/scout_batch_49/loc.py`: non-blank, non-comment lines. "Stripped" is the original as if C++ already had a perfect contest library: without the template that opens every file, the stream setup lines, the debug printers, and the stock data structures (the sparse table in `ioi99p6`, the segment trees in `ioi21p1`, `ioi22p4` and `ioi18p2`, the union-find in `ioi12p2`, the prefix sums in `ioi17p2`, the modular power in `ioi14p4`). Problem-specific node types stay in.

```
problem      raw stripped   v2 today
ioi00p2       85       58   28    37
ioi01p3      173      145   46    53
ioi99p6      104       56   22    40
ioi14p4      110       75   30    55
ioi15p5      166      130   28    57
ioi17p2      134      101   24    50
ioi21p1      128       41   25    63
ioi22p4      129       51   34    63
ioi12p2      158      111   42    67
ioi18p2      158       91   41    74
total       1345      859  320   559
```

The v2 ports are 76% shorter than the raw C++ and 63% shorter than the stripped C++. The buildable forms are 58% and 35% shorter.

Two things to read into those numbers. First, part of the v2 saving is algorithmic, not linguistic: `ioi17p2` (101 to 24) and `ioi15p5` (130 to 28) are mostly a simpler method, and `ioi01p3` (145 to 46) removes a duplicated solver. The cleanest language-only comparisons are `ioi21p1` (41 to 25), `ioi22p4` (51 to 34) and `ioi18p2` (91 to 41), where the stripped original has already lost its data structure. Second, the 239-line gap between `v2` and `today` is the largest of any batch so far, and three files account for most of it: `ioi21p1`, `ioi22p4` and `ioi18p2` each write a segment tree out by hand because the prototype has no custom-monoid descent and no custom `Action`. In those three the `today` form is longer than the stripped C++.

## 1. Friction and collisions in the v2 design itself

These remain even if every cheat-sheet feature is implemented. They are ordered by how much they cost while writing.

### 1.1 A stateful `@grader struct` cannot name the domains its fields are indexed by

**Hit in:** `ioi12p2.cp`, `ioi18p2.cp`, `ioi22p4.cp`. Extends batch 37 item 1.1 from functions to structs.

All three are "init once, then answer calls" tasks. The state is arrays over rings, seats or gates, whose sizes arrive as arguments of the first call. The sheet gives `@grader struct` one word ("stateful") and no example. Writing the struct, the fields need a domain before any function has run:

```cp
@grader
struct Hall:
    domain Row, Col, Who             // declared here ...
    seat: [Who]Cell
    who: [Row, Col]Who
    bad: LazySegTree<Who, MinCount>

    fn give_initial_chart(h: Int, w: Int, r: [Who]Row, c: [Who]Col) -> Hall:
        domain Row = 0..h            // ... and sized here
        domain Col = 0..w
        domain Who = 0..h * w
```

I invented a bodiless `domain Row, Col, Who` in the struct and a sizing declaration in the constructor. Nothing on the sheet says this is legal, and it collides with two other rules. A helper struct declared outside (`Without` in `ioi12p2`, whose fields are `[Ring]Int` and `Dsu<Ring>`) uses `Ring` at file scope, where it does not exist. And the later calls take `Who` and `Ring` arguments (`swap_seats(a: Who, b: Who)`), so the grader boundary has to convert the judge's `int` into a domain value that was created by an earlier call.

This is the single biggest design hole for IOI-style tasks: three of ten problems in this batch, two of ten in batch 37. Proposal: one stated rule, for example "a `@grader struct` may declare `domain D` fields; exactly one function (the constructor) fixes them with `domain D = ...`; every other file-scope use of `D` refers to that instance", with the three-line example on the sheet.

### 1.2 `LazySegTree` with a custom `Action` does not say where the monoid goes

**Hit in:** `ioi22p4.cp`, `ioi18p2.cp`.

The sheet shows `LazySegTree<D, T>(AddSum, a)` and says a custom tag is `Action(id, compose, apply)`. Both problems need a custom value monoid as well: a pair sum with a flip, and (minimum, count of minimum) with an add. The six stock tags bundle the monoid with the action, so the sheet never shows the four-part form. I followed batch 45's guess:

```cp
let flip = Action(false, (f, g) => f != g, (f, x) => if f: (x.all - x.on, x.all) else: x)
LazySegTree<Source, Lit>((0, 0), (a, b) => (a.on + b.on, a.all + b.all), flip, leaves)
```

Two ports now depend on an argument order that is written down nowhere. Proposal: put the full signature `LazySegTree<D, T>(id, op, action, a)` on the sheet next to the stock form.

Separately, "range add, count of the minimum" is the second most common lazy tree after `AddMin` (Seats, area of union of rectangles, every "how many positions are zero" sweep). It took a named tuple, a 2-line `both` function and an `Action` here. A stock `AddMinCount` tag would make the `Hall` constructor one line shorter than the stock ones.

### 1.3 `min_left` / `max_right`: boundaries or indices

**Hit in:** `ioi21p1.cp`. Batch 45 item 1 asked what `max_right` returns; this is the mirror case, and here the answer changes the algorithm's shape.

Candies needs the latest day `p` such that the walk over days `p..` spans the box. With a boundary-returning `min_left` that is clean:

```cp
let tail = days.min_left(Day.last.r, w => w.span() < cap[box])
let after = days.fold(tail..)
ans[box] = if let p = tail.left and v[p] > 0: cap[box] - (after.top - after.end)
           else: after.end - after.low
```

`tail` is a gap, `tail..` is every day after it, `tail.left` is the day that broke the predicate, and `none` falls out when no day does. If `min_left` instead returns an index, all three uses need a `+ 1` / `- 1` and an emptiness check. I assumed boundaries (`to` is a `V.bounds`, the result is a `V.bounds`), because the sheet's boundary vocabulary makes that the only version with no arithmetic. The sheet should say so.

### 1.4 `@memo` over a small array, and `@memo` that closes over changing state

**Hit in:** `ioi01p3.cp`. Two collisions in one function.

The Twofive state is five row lengths, each `0..=5`. The sheet rules out `[D]E` as an `Index`, so the key is a 5-tuple, which cannot be looped over or updated in place:

```cp
type Shape = (0..=5, 0..=5, 0..=5, 0..=5, 0..=5)
@memo
fn ways(shape: Shape) -> Int:
    let len = [shape.0, shape.1, shape.2, shape.3, shape.4]
    ...
    var grown = len.copy()
    grown[r] += 1
    total += ways((grown[0], grown[1], grown[2], grown[3], grown[4]))
```

Three of the function's eleven lines convert between tuple and list. `[0..5]0..=5` has 7,776 values and a trivial O(1) mixed-radix index; the exclusion on the sheet reads as a blanket rule against arrays, when the real constraint is "small constant length". Proposal: allow `[C]E` as an `Index` when `C.K` is a compile-time constant and `E.K ** C.K <= 2^26`.

The second collision: the count is recomputed 300 to 600 times with a different set of pinned letters. I nested the `@memo fn` inside `fn completions(pin)` and assumed the table lives for one call of the enclosing function. The sheet says only "flat table for algebraic Index"; batch 28 found that the prototype requires `@memo` at top level. If the table is global, this program is silently wrong (stale counts from the previous `pin`). The rule needs stating: a nested `@memo` that reads enclosing parameters gets a fresh table per call of the enclosing function. Without that the only correct spelling is the hand-rolled `memo.fill(-1)` of the `today` form.

### 1.5 A `Perm` and its inverse, kept in step

**Hit in:** `ioi15p5.cp`.

Every phase of Sorting swaps two slots and needs the inverse permutation afterwards. `Perm<V>` has `.inv()` and `.cycles()`, which made the feasibility test one line (`Slot.K - row.cycles().len <= t`), but nothing maintains the pair:

```cp
fn exchange(a: Slot, b: Slot):
    swap(mut row[a], mut row[b])
    swap(mut slot_of[row[a]], mut slot_of[row[b]])
```

The second line is the classic place to get the order wrong (it must read `row` after the first swap). The same two lines appear in every "sort by swaps" problem. Proposal: a local-library `PermPair<V>` (or `Perm.tracked()`) with `.swap(a, b)`, `.at(slot)` and `.find(value)`.

The prototype added a second, borrow-flavoured friction here. `swap_slots(v, slot_of[v])` is rejected because `swap_slots` mutates `slot_of`:

```
error: line 59: `slot_of[v]` can't be passed to `swap_slots`: `swap_slots` also changes `slot_of` directly
```

`slot_of[v]` is a `Copy` integer evaluated before the call, so there is no aliasing. v2's ownership section says small values are `Copy`; the check should only fire for `mut` arguments and heap values. The message itself was clear and the fix (bind it first) took seconds.

### 1.6 Elements of a `List<struct>` cannot be mutated in a `for`

**Hit in:** `ioi12p2.cp`.

The four candidate graphs are a `List<Without>`, and every `Link` updates all four. `for c in self.cands: c.link(u, v)` would mutate copies (or be rejected, the sheet does not say), so the loop goes through positions:

```cp
for i in 0..self.cands.len: self.cands[i].link(u, v)
```

This is the one place in the batch where I wrote an index loop purely for the language. `for mut c in xs` or `for c in mut xs` is the obvious missing spelling; the sheet already has `mut` at parameters and call sites, so it costs no keyword.

### 1.7 Carving one domain out of another

**Hit in:** `ioi22p4.cp`, `ioi99p6.cp`. Related to batch 37 item 1.3 (a prefix of a domain).

Digital Circuit numbers gates `0..n` and sources `n..n+m` in one parent array, then indexes a segment tree by source. `Source` is a suffix of `Node`, and the port crosses between them three times with raw arithmetic:

```cp
let w = worth[Node.from_idx(n + s.to_idx)!]
self.lit.apply(Source.from_idx(l - self.gates)!..=Source.from_idx(r - self.gates)!, true)
```

Strip of Land has the smaller version: "at most 100 columns starting at `left`" is `(left.to_idx..left.to_idx + 100).clamp(Col)`, out to `Int` and back. Proposal: `domain Source = Node[n..]` (a named sub-range with `.up` / `Source.of(node) -> ?Source` conversions), and a `.take(k)` on ranges so the window is `(left..).take(100)`.

### 1.8 Smaller items

- **No `product(it)`** (`ioi22p4.cp`). `sum`, `any`, `all`, `count` exist; the subtree count is a two-line loop instead of `all[u] = c * product(all[v] for v in kids[u])`. With `Mod<P>` this comes up in every counting DP on trees.
- **"All but one" products without division** (`ioi22p4.cp`). The modulus `1_000_002_022` is not prime, so the prefix-times-suffix trick is written by hand (7 lines). A `prefix`-like `excluding(xs, *)` yielding the fold of everything but position `i` would cover this and the rerooting pattern.
- **Nearest earlier element with a property, from both sides** (`ioi17p2.cp`). "Distance to the closest point of the other colour" is a small nested `fn sweep(order: Range<P>)` called twice. It reads fine, but I first wrote `P.first..=P.last` for the forward order because a bare domain is not a `Range<P>`; `P.all` or accepting `P` where a range is expected would remove the hesitation.
- **First strict maximum with its witness** (`ioi99p6.cp`). The answer must be the first rectangle of maximum area in scan order (ties matter for matching a reference). `max_by` does not state which maximum it returns, so the port keeps `best_area` and `best` side by side. One sentence on the sheet ("`max_by` and `min_by` return the first") would allow `max_by(candidates, c => c.area)`.
- **`slide_until` on unextendable starts** (`ioi99p6.cp`). The sheet does not say what is yielded for a left end where even the single-element window fails. The port assumes such starts are skipped.
- **Advancing an optional cursor** (`ioi00p2.cp`). `cursor[car]` is a `?Slot` because `v + 1` is optional, so "skip forward while the slot already holds this type" is `while park[cursor[car]!] == car: cursor[car] = cursor[car]! + 1`, three unwraps for a pointer walk that cannot run off the end. This is the same shape as batch 37 item 1.5.
- **Grader arrays sized by a separate argument** (`ioi14p4.cp`). `valid(n, seq)`, `replacement(n, seq)` and `countReplacement(n, seq)` each spell `seq: [0..n]Int`; as in batch 37 item 1.2 the length is known twice.

## 2. Gaps between the cheat sheet and `proto/cpc`

Each cost lines in a `today/` file. None needed a change to `proto/cpc`.

- **No custom `Action`, no `min_left`.** `ioi21p1`, `ioi22p4` and `ioi18p2` each carry a 25 to 30 line recursive tree over parallel `List<Int>`s. This is most of the `v2`/`today` gap.
- **`var (a, b, c) = (0, 0, 0)` at top level is not visible inside a `fn`.** The emitted C++ failed with "`tail_end` was not declared in this scope" (`ioi21p1`); three separate `var` lines work. Inside a loop body the same form builds and mutates correctly (`ioi99p6`).
- **`Str` is opaque.** Twofive prints a word built letter by letter. `Str.push`, `Char(65 + i)`, `"".join(...)` and `s[i..j]` are all rejected; a `List<Char>` printed with `print` comes out as a word, and `'A' + i` yields a `Char`, so that is what `today/ioi01p3.cp` does.
- **`6 ** (4 - r)` as a divisor is rejected** ("may be 0") although both operands are positive constants and a loop index in `0..5`. The port uses a literal weight table.
- **Records and tuples in lists.** To stay clear of the known destructuring and tuple-assignment miscompiles (batches 12, 17, 20), move lists and swap lists are flattened to `List<Int>` with `2 * i` arithmetic in `ioi00p2`, `ioi15p5`, `ioi12p2` and `ioi18p2`.
- **No `Perm`, `SparseTable`, `slide_until`, `Dsu` inside a struct, `Set` of cells, `adj4`.** Hand-written inverse arrays, two index-based monotone queues (`ioi99p6`), a flat five-layer union-find (`ioi12p2`), and sort-then-skip-duplicates for the touched cells (`ioi18p2`).
- **Whole-program form only.** There is no `@grader`; every grader problem reads a stdin format invented for the harness.

## 3. What read well

- **Boundaries.** `cost: [P.bounds]Int` with `cost[p.r] = cost[p.l] + near[p]!` in Wiring, and `stops[r[d].r]` / `stops[box.l]` in Candies, removed every `+ 1` from two prefix-indexed sweeps. The C++ originals have `dp[n + 1]`, `psum_c[i + 1]` and `upds[qr[i] + 1]`.
- **Optional `min`.** The Seats corner rule is "until the first of my upper and left neighbours arrives", where either may be off the grid. `min(self.arrival(up), self.arrival(left))` skipping `none`, with `fn arrival(self, q: ?Cell) -> ?Who: self.who[q?]`, replaces the original's `n * m` and `1e9` sentinels.
- **A struct as a monoid.** `Walk` with `then` and `span` in Candies is the whole idea of the solution in six lines, and `SegTree<Day, Walk>(Walk(), (a, b) => a.then(b))` needs nothing else.
- **`bsearch_first` over a closure that replays state.** Sorting's outer search is one expression.
- **`Mod<P>` with a composite modulus.** Digital Circuit never writes `% MOD`, and the type makes it visible that no `.inv` is used.

## 4. Status against v2.1

This batch was written against the v2 sheet at `3723dca`, as assigned. The v2.1 sheet landed on `main` while it was in progress. Reading the new sheet against section 1 (nothing was re-run, and the ports were left in v2 spelling):

- **Resolved by v2.1:** `for mut b in blocks` (1.6); `prod(it)` (1.8); "`max_by` ... first extremum wins ties" (1.8); `@memo` "nested in loops (block-scoped cache)", which is the lifetime 1.4 asked for; `Set<T>(it)` from an iterable, which would shorten `swap_seats` in `ioi18p2.cp` by three lines.
- **Partly addressed:** 1.1. The v2.1 sheet says a file-scope `domain V` "binds from grader args for shared callbacks", which covers batch 37's function case. It still shows no `@grader struct` whose fields use a domain fixed by its constructor, and no helper struct sharing that domain. 1.2: `Action(id, compose, apply, over:)` now has a fourth argument, but the sheet still shows no call with a custom value monoid, so the argument order used in `ioi22p4.cp` and `ioi18p2.cp` remains a guess. 1.7: `V[..k]` gives a prefix of a domain, not a suffix (`Node[n..]`) or a window from a position.
- **Still open:** 1.3 (boundaries or indices for `min_left` / `max_right`), the array-as-`Index` half of 1.4, 1.5 (`Perm` with its inverse), "all but one" products, the optional cursor walk, and `slide_until` on unextendable starts.
- **New in v2.1 that touches a port:** `SparseTable.fold` now returns `?T`, so the predicate in `ioi99p6.cp` (`peak.fold(t..=b) - pit.fold(t..=b) <= k`) would need two unwraps on ranges that are never empty.

## 5. Ranked recommendations

Items already settled by v2.1 are left out.

1. Specify `@grader struct` with domains fixed by its constructor and shared with helper structs (1.1). It blocks the cleanest spelling of every init-then-query IOI task.
2. Show one `LazySegTree` call with a custom monoid and a custom `Action` on the sheet, and add an `AddMinCount` stock tag (1.2).
3. State that `min_left` / `max_right` take and return boundaries (1.3), together with batch 45's request for the return type.
4. Admit small constant-length arrays as `Index` (1.4).
5. Add suffix and window sub-domains (`Node[n..]`, `(left..).take(100)`) beside `V[..k]` (1.7).
6. Keep `SparseTable.fold` total on non-empty ranges, or give it a non-optional form; the v2.1 change costs unwraps in the commonest use.
7. Local library: `PermPair` (1.5), `excluding(xs, op)` (1.8).
8. Prototype: make top-level tuple `var` global, relax the `Copy`-argument alias check, and give `Str` `push` and indexing (section 2).

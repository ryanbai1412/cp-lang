# Friction backlog, batch 13: binary search, two pointers, divide and conquer

A Stage-1 scout pass over ten submissions: four searches for an answer
(one interactive), one two-pointer problem, two that stopped needing a
search once rethought, one divide and conquer over convex lists, one
divide-and-conquer DP, and one interactive hunt for a local maximum. The
ports are in
[`proto/ports/scout_batch_13/`](../proto/ports/scout_batch_13/):
coci12c1p4, coci21c4p4, cpc21c1p5, tle17c5p6, ioi11p3, ceoi17p2, ioi05p4,
apio07p2, ioi14p6 and btoi18p3. Items are numbered `R1`... so they do not
collide with `F` (batch 1), `G` (3), `B` (2 and 4), `K` (5), `S` (6), `D`
(7), `H` (8), `T` (9 and 10), `N` (11), `L` (12) or `P` (14).

What was checked: seven ports have a `today/` form that builds with
`proto/cpc` and agrees with its original on 100 random cases
(`proto/ports/scout_batch_13/check.sh`) on top of commit `f5030cd`.
ceoi17p2, ioi14p6 and btoi18p3 were not built, and the ioi14p6 sketch was
not checked in any way. No port was run at full size. Every "today"
snippet is a line of a `today/` form or of a probe that was compiled.
Every quoted message was produced by `proto/cpc`. The "wanted" snippets
do not build. A count "of 10" is the number of the ten ports with the
awkward line. A corpus count is the number of problems, out of the 710
with an accepted C++ submission, whose latest source matches a regular
expression; treat it as a rough size, not a measurement.

Two ports written for this batch, coci10c7p4 and dmopc18c1p6, were
dropped before the push: batches 12 and 14 landed them first. What they
showed is under R3 and R6. No CDQ submission is in the corpus (no
accepted source contains `cdq`), and the only ternary-style search is the
golden-section part of btoi18p3, so those two themes are thin here.

## Suggested order for a Stage-2 sprint

1. R1 and R2: finish the search family in userland. 4 of 10.
2. R4: a type error that prints the same type twice.
3. R3: windows whose test is the window; one loop that was written wrong
   twice.
4. R5: inputs that come with a promise. 5 of the 7 `today/` forms carry a
   hand-written generator for one.
5. R6, R7, R8: the helpers behind DP optimisations and divide and conquer.
6. R9, R10, R11: real numbers, questions to a judge, rectangles.

## R1. The search family is half there (4 of 10; batch 5 K6, batch 7 D8)

Good news first. *Checked on `f5030cd`:* `(1..=1e9).first_where(...)`,
`(0..=2e14).first_where(...)` and `(0..=2e14).first_where(big)` with a
named function all build and run, so K6 is closed for "the first".
`today/coci12c1p4.cp` is one line of search, and `today/coci21c4p4.cp`
searches a range that no domain could hold.

There is still no "last one that works". *Checked:* `(1..=n).last_where(...)`
is "Range<Int> has no method `last_where`" and `S.last_where(...)` is
"type S has no method `last_where`". cpc21c1p5 needs it twice (the
greatest left edge that still holds a rectangle) and tle17c5p6 once, and
both turn it into a first on the negation, with a fallback for "all of
them work" and a `- 1`:

```cp
let x1 = ((1..=x2).first_where(x => left(x, x2, 1, 1e9) == 0) ?? x2 + 1) - 1
let flip = (0..=n).first_where(k => light(k) != dark)! - 1
```

The original of cpc21c1p5 has the same trick as a template (`last_true`
calls `first_true` on the negation and subtracts one). The midpoint of
two positions is label arithmetic (*checked:* "Range<S> has no `.mid`"):

```cp
let mid = Field((l.label + r.label) / 2)        // today/ioi11p3.cp: the hub goes on the middle field
```

Wanted:

```cp
let x1 = (1..=x2).last_where(x => left(x, x2, 1, 1e9) > 0)!
let hub = w.mid
```

Proposed: one loop in a userland `stdlib/search.cp`, generic over what a
`Range` can hold (an `Int` or a position), and the rest defined from it,
so that nothing about searching is in the compiler:

```cp
pub fn partition_point<T>(r: Range<T>, pred: T -> Bool) -> Bound<T>     // the only loop; it calls r.mid
pub fn first_where<T>(r: Range<T>, ok: T -> Bool) -> ?T = partition_point(r, x => not ok(x)).right
pub fn last_where<T>(r: Range<T>, ok: T -> Bool) -> ?T = partition_point(r, ok).left
```

The plan (12 §7.2) still calls these `first_true` and `last_true`; the
prototype's `first_where` on a range reads better and should be the one
name. Counts: 4 of 10 search, 2 of them need `last_where`, and 2 more
ports use a midpoint. In the corpus 46 problems have the hand loop and 12 define
their own `first_true` or `last_true`.

## R2. A search that keeps what the test found (coci21c4p4)

The test "can k parks reach distance x" builds the list of parks. The
search returns only x, so the parks are built a second time. The original
does the same (`check(l);` after the loop), and so does
`today/coci21c4p4.cp`:

```cp
let reach = (0..=2e14).first_where(x => parks(x).len <= k)!
print(reach)
var chosen = parks(reach)
```

Wanted: the test returns an optional, and the search returns the first
value that gave one, with what it gave.

```cp
fn parks(reach: Int) -> ?List<Town>: ...      // none if it takes more than k
let (reach, built) = (0..=2e14).first_some(parks)!
```

Proposed: `first_some<T, W>(r: Range<T>, f: T -> ?W) -> ?(T, W)` and
`last_some`, in the file of R1. `first_where(r, ok)` is then `first_some`
with an empty payload, so there is still one loop. It also lets the test
stop early: `parks` returns `none` at the k + 1st park instead of
finishing the tree. 1 of 10. Two corpus problems run the test again right
after the loop (coci21c4p4, mmcc14p3); problems that keep the witness in
a global written by the test were not counted.

## R3. A window whose test is the window (ioi11p3; extends batch 12 L8)

The plan's `sliding` (12 §6.5) threads a state through `add` and
`remove`. ioi11p3 has no such state: the cost of a block of fields comes
from prefix sums in O(1). *Checked:* `sliding` is "unknown name sliding"
anyway, so `today/ioi11p3.cp` moves the ends by hand:

```cp
var best = 0
var l = Field.first
for r in Field:
    while cost(l, r) > budget: l = l.next!
    best = max(best, r - l + 1)
```

Batch 12 L8 has the mirror image (coci10c7p4: for each lowest height, the
least highest one that still delivers) and proposes `stretches`. That
loop is easier to get wrong than it looks. This batch ported coci10c7p4
too, before batch 12 landed, and its first form printed `-1` for a town
with no house: every span of heights "delivers" then, the empty ones
too, so the upper end fell behind the lower one. It needed

```cp
hi = max(hi, lo)                // with no house every window delivers, the empty ones too
```

`proto/ports/scout_batch_12/coci10c7p4.cp` has the loop without that
line. *Checked:* on the 2 x 2 town `..` / `P.` with heights `2 1` / `2 1`
it prints `-1` and the original prints `0`. Two sessions wrote the same
loop and both got it wrong, which is the case for a library function.

*Fixed.* The batch 12 port has the line now, and its check draws towns
with no house.

Wanted:

```cp
print(max(w.len for w in sliding(Field, ok: w => cost(w) <= budget)))
```

Proposed: one `sliding` for both directions. `ok:` receives the window
(and the state, when there is one) and is only ever asked about a
non-empty window, so `cost` can take a `Range1`; `init`, `add` and
`remove` become optional. `shortest:` is L8's `stretches` under the same
name: for each left end the shortest window that passes, never an empty
one. Debug runs check the monotonicity at the windows they visit, as the
plan already says. 1 of 10 here, 2 of 10 in batch 12.

## R4. "expected X, got X" (message; apio07p2; batch 4 B9)

```cp
fn sorted_offices() -> [n]0..=1e9:
    var at: [n]0..=1e9 = 0
    at
```

*Checked:* "expected [0..n]Int[0..=1000000000], got
[0..n]Int[0..=1000000000]". This is B9 (two tables over the same written
range are different types) with a message that prints one type twice.
The port needs `type Offices = [n]0..=1e9`. Proposal: fix B9, so that two
tables over the same range expression of the same immutable name are one
type. Until then the message should say that each `[n]` is its own index
space and suggest a `type` alias.

## R5. Inputs that come with a promise (5 of 10; extends batch 6 S4)

The statement promises sorted positions (ioi11p3, apio07p2), a
permutation (ioi05p4), a tree (coci21c4p4) or earlier wires only
(tle17c5p6). Five of the seven `today/` forms carry a generator for the
promise alone:

```cp
fn sorted_fields() -> [Field]1..=1e9:
    var at: [Field]1..=1e9 = 1
    for i in Field: at[i] = rand(1, len)
    at.sort()
    at

let x: [Field]1..=1e9 = read(gen: sorted_fields)
```

Batch 6 S4 proposes `read(increasing)`, `read(distinct)` and
`read(covers: V)`; batch 5 K19 a `Span<D>`; batch 8 H6 a `Perm<V>` that a
value indexes. Wanted here:

```cp
let x: Sorted<[Field]1..=len> = read()
let roads: Tree<Town, (w: 1..=1e9)> = read()
```

Proposed: one mechanism instead of a list of shapes. A sealed struct may
declare how a value of it is generated and what must hold of one that is
read, and `read()` uses both:

```cp
pub struct Sorted<A>(sealed items: A):
    @gen fn random(raw: A) -> Sorted<A> = Sorted(sorted(raw))      // the generator makes an A, this repairs it
    law ordered = all(a <= b for (a, b) in items.adjacent)         // checked on read in local runs
```

`Sorted`, `Span`, `Perm`, `Distinct` and `Tree` are then library types
that anyone can add to, and S4's `read(increasing)` is `Sorted` with a
stricter law. A `Sorted` value is also the natural home for `lower_bound`
(D8), because the promise the search relies on is in the type. Simplifies
5 of 10 here plus S4, K19 and H6.

## R6. A best choice fenced in by its neighbours (ioi14p6; relates batch 14 P13)

ioi14p6 computes, for every number of days t, the best last city i. With
more days the best last city never moves back, so the original fills the
middle t first and hands each half the range of cities that is left, 14
lines:

```cpp
void dp(int l, int r, int ql, int qr) {
    if (l > r) return;
    int mid = (l + r) / 2;
    pair<long, int> opt {-1, -1};
    for (int i = ql; i <= qr; i++) opt = max(opt, pair<long, int> {ds.query(i, mid - (i + 1) * mul), -i});
    ...
    dp(l, mid - 1, ql, qmid);
    dp(mid + 1, r, qmid, qr);
}
```

dmopc18c1p6 (batch 14, and dropped here) is the interval form: the best
cut of a block lies between the best cuts of the two wider blocks, kept
in a second table. P13 proposes `min_by` for the inner loop of both.
Wanted: say what a choice is worth and let the library keep the fence.

```cp
dnc_opt(0..=days, side.keys, (t, i) => top.sum(..=i, largest: t - (i + 1) * step))
fix = knuth(Word, (zeros, rest) => ones.sum(zeros) + (rest.len - ones.sum(rest)) + fix[zeros] + fix[rest])
```

Proposed, both userland: `dnc_opt(ts, choices, value) -> [ts]Int` (the
best value for each t, given that the best choice is monotone in t) and
`knuth(D, cost: (Range<D>, Range<D>) -> Int) -> [Range<D>]Int` (the
cheapest way to cut every block in two). The plan lists both as names
only (20 §G). They should be built on one primitive, "the best of a range
that two earlier answers bound", and debug runs should check the
monotonicity against a full scan on small inputs, as `sliding` does. The
second needs a table with one cell per block of a domain, `[Range<D>]T`,
which is batch 1 F6 (windows as an index type) for every length at once.
With blocks named by their bounds, dmopc18c1p6's three cases (all ones,
all zeros, a cut inside) are one formula, because an empty part costs
nothing. 1 of 10 here, 1 of 10 in batch 14. A search for `optl`, `optr`
or an `opt` table matches 14 corpus problems, but several of those are
segment trees: ioi14p6 and dmopc18c1p6 are the two that were read.

## R7. Divide and conquer with an associative join is a fold (apio07p2)

The port joins the answers of two neighbouring runs of gaps, and the
join is associative. The recursion exists only to pair up neighbours of
similar size:

```cp
fn run(l: Int, r: Int) -> List<Costs>:
    if r - l == 1: return [[0, at[r] - at[l]], [0], [0], [0]]
    let mid = (l + r) / 2
    join(run(l, mid), run(mid, r))

print(run(0, n - 1)[0][k])
```

`l`, `r` and `mid` are list offsets, and `r - l == 1`, `n - 1` and
`at[r] - at[l]` are three chances to be off by one. Wanted:

```cp
let whole = fold_balanced(join, (one(q - p) for (p, q) in at.adjacent))
```

Proposed: `fold_balanced(op, xs)` next to `reduce` (12 §7.1): the same
result as a left fold for an associative `op`, with the brackets
balanced, so that a join whose cost grows with its arguments costs O(n
log n) in total instead of O(n^2). It takes the `Monoid` or `Semigroup`
value that `reduce` takes, so the law that makes it correct is already
stated. Merge sort, the product of n small polynomials and this port are
the same call with a different `op`. 1 of 10. A narrow search for a
recursion on `(l, mid)` and `(mid, r)` whose results are joined finds 2
corpus problems (apio07p2, cco18p6).

## R8. Convex cost lists, and two lists walked together (apio07p2, ceoi17p2)

`costs[j]` (the cheapest j cables in a run) is convex in j. Splitting j
between two runs is a min-plus product, which for convex lists is a merge
of their steps. `today/apio07p2.cp` writes the merge on raw offsets, 11
lines, and the pointwise minimum of two lists of different length, 7
more:

```cp
while i < a.len or j < b.len:
    if j == b.len or (i < a.len and a[i] - a[i - 1] <= b[j] - b[j - 1]):
        out.push(out[out.len - 1] + a[i] - a[i - 1])
        i += 1
    else:
        out.push(out[out.len - 1] + b[j] - b[j - 1])
        j += 1
```

*Checked:* `merge` is "unknown name merge". Wanted: `a.min_plus(b)` and
`min(a, b)` on a `Convex` list. Proposed: a userland `Convex` (a sealed
list whose steps do not decrease, in the sense of R5) with `min_plus` and
`min`, written with three pieces that are useful alone: `steps(xs)`, the
inverse of `prefix`; `merge(a, b)` of two sorted sequences (12 §6.6 lists
it); and `zip_longest(a, b)`. Then `min_plus` is `prefix(merge(steps(a),
steps(b)), from: a[0] + b[0])`.

ceoi17p2 walks two sorted lists and takes from whichever has paid less so
far. That rule depends on running totals, so `merge` does not cover it,
and the sketch keeps the loop with two hand-moved places. No proposal:
one port is not enough to design a cursor from. 1 of 10 for `Convex`, 2
of 10 for the two-place loop.

## R9. A search over real numbers (ceoi17p2; extends batch 14 P4)

*Checked:* `Float` is "unknown type Float" (P4, B5), so ceoi17p2 has no
`today/` form. The original searches the answer by halving `[0, 1e18]`
200 times. The port does not search, but a real-valued search will come
up again, and `first_where` on a range of reals has no last step to stop
at. Wanted when it is needed: `bisect(0.0..=1e18, ok, steps: 200)`, the
loop of R1 with the number of halvings in the call, and a range type of
reals for input (`a: 1.0..=1000.0`) so the generator and the checks work
as they do for integers. 1 of 10; 2 corpus problems halve a `double`.

## R10. Questions to a judge (2 of 10: btoi18p3, cpc21c1p5; batch 5 K7, K21)

A question asked twice should be asked once. btoi18p3 has a budget and
asks about the same cell from several places; the original keeps a cache
by hand. *Checked:* `@memo` on a function whose body is `let h: 0..=100 =
ask("?", x)` builds and asks once. But `plan/15` §3.3 and decision 5.9
say `@memo` on a function that does I/O is an error. Proposal: keep the
error for `read()` and `print`, and allow `ask`: the judge's reply is a
function of the question, which is what `@memo` needs.

`ask` only works as the whole right side of a typed `let`. *Checked:*

```cp
fn left(x1: Int, x2: Int, y1: Int, y2: Int) -> Int:
    ask("?", x1, x2, y1, y2) - count(...)        // error: unknown name ask
```

Proposal: `ask` is an expression wherever its reply type is known, from
the function's return type or an operand, and elsewhere the message says
that a type is missing, not that `ask` is unknown.

A local maximum of hidden numbers. The first third of btoi18p3 is 28
lines of golden-section search, which keeps one of its two probes for
the next round. Wanted: `local_max(X, x => height(x))`, a position whose
value is not below either neighbour's. This is not K7's `argmin_convex`:
the heights are not unimodal and the search only promises a local
answer. Proposed: `local_max` and `local_min` on any range, with the
number of calls in the documentation (about 1.44 log2 n), since in an
interactive problem that number is the budget (K21). 3 corpus problems
have a golden-section or ternary search.

## R11. A rectangle, cut along its longer side (btoi18p3; batch 5 K14, batch 12 L5)

The second third of btoi18p3 halves a sheet along its longer side, scans
the middle line, and continues in one half. The original has the whole
step twice, 18 lines for a vertical cut and 18 for a horizontal one,
differing only in which coordinate is which. Wanted:

```cp
type Sheet = Range<(X, Y)>
let (line, low, high) = sheet.halve()
let (before, after) = line.across(c)
```

Proposed: a range of a product domain is a rectangle (K14 already
multiplies ranges, and L5 makes a pair of positions a position), with
`halve()` (the middle line of the longer side and the rectangles either
side) and `across` (the cells beside a cell of the line, one in each
half, if they exist). Both are userland over an `Axis` enum and
`p.with(axis, v)`; the axis is a value, so nothing is written twice.
1 of 10.

## Smaller notes

- tle17c5p6 evaluates a circuit once per probe. Today the memo is a
  top-level table that `light` clears and a function that fills it, 9
  lines. Wanted: `@memo` on a function nested in `light`, with a table
  that lives for one call. 1 of 10.
- *Checked:* `"0" * n` is "`*` needs Int operands, got Str and
  Int[1..=9]" and `"".join(...)` is "Str has no method `join`".
  `today/tle17c5p6.cp` fills a `[n]Char` and prints that.
- *Checked:* `v[w] = if c: true else: none` is "`none` needs a known ?T
  type here", though the slot's type is known.
- *Checked:* `Int.MAX` is "type Int has no `.MAX`", though the tour (10)
  uses it. `var best: ?Int = none` with `best min= x` reads better.
- ioi14p6 needs the sum of the k largest of a prefix. The original has
  45 lines of square-root blocks. A persistent segment tree with a walk
  would give it; nothing in `stdlib/` does yet.
- `proto/interact.py` dropped the rest of a solution's output when the
  solution had already exited and the judge still replied to one of its
  lines (cpc21c1p5 prints `!` and then n lines; one `check.sh` run in
  four judged it wrong). Fixed in this batch: a reply that cannot be
  delivered no longer ends the conversation.
- The search in coci21c4p4 calls a function that walks the whole tree
  about 48 times. A cost estimate that multiplies the two would be a
  useful warning.

## Seen again from earlier lists

- Batch 6 S5: `var at: [Field]1..=1e9 = [Field] i => rand(1, len)` still
  builds C++ that `g++` rejects. ioi11p3 hit it twice.
- Batch 6 S17, batch 12 L18: `xs.adjacent` is "List<Int> has no
  `.adjacent`", `xs ++ [x]` is "expected an expression (at '+')", and
  `a.rev` as a value is "has no `.rev`" (K10). ioi05p4's widest gap on a
  circle is a loop with the wrap-around beside it, and its mirrored
  seating is `[Seat] i => wish[Seat(n + 1 - i.label)]`. The sketch
  appends `shifts[0] + n` and takes `.adjacent`; a `.cyc` view would not
  help, because the closing pair needs `+ n` on one side.
- Batch 12 L1, L7: btoi18p3 wants `c.around(Dir6)`.
- H1, S4: `let x: [Field]1..=len = read()` produced `4` with `len = 2`.
- H6: `wish[i]` on a `Perm<Seat>` is "index of type Seat into array over
  Pos<Seat>".
- B14: coci21c4p4 keeps the road lengths in an `up` table beside
  `Rooted`.
- B11: `[Bool, Bool]Costs` is "array dimension must be a domain, a
  range, or a struct with `impl Index`, got Bool"; apio07p2 uses a list
  of four and `2 * l + r`.
- K11: tle17c5p6's wires are inputs then gates, told apart by
  `w.label <= n` and `Gate(w.label - n)`.
- F1, H3: `std.math.ceil_div` and `std.tree.Rooted` cannot be imported
  by a port that `proto/cpc` builds.

## Already good

- `first_where` on a plain range, 64-bit ranges included, with a lambda
  or a function name, and `?? x` for "none works".
- `if let nr = r + dr and let nc = c + dc and not seen[nr, nc] and lo <=
  height[nr, nc] <= hi:` builds: steps off a grid, a chained comparison
  and a test in one condition.
- `var best: ?Int = none` and `best min= x`: no sentinel.
- `read(gen: f)`, and `proto/interact.py` with a short Python judge.
- `Rooted(adj, root)` with `tree.preorder.rev` gives "children before
  parents" without recursion, so the greedy of coci21c4p4 is a flat loop.
- `print(b)` for a record prints its fields in order, and a `[n]Char`
  prints as a word.
- `domain Reach = 0..=2e14` is refused with a message that names the
  limit and a way out.

## An original that is not a safe oracle

`5245322.cpp` (coci12c1p4) starts its search at 0 and divides by the
midpoint. On the input `1 1 1`, where the answer is 1, it divides by zero
and prints nothing. `today/coci12c1p4.cp` rejects inputs whose answer is
1 with a `where`, which is why two thirds of its generated cases are
thrown away.

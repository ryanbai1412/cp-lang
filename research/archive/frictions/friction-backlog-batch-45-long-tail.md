# Friction backlog, scout batch 45: hard long-tail corpus classics

Ten accepted C++ submissions from unclaimed prefixes (`shoi`, `ucrpc`, `iati`,
`neerc`, `rpeng`, `noimps`) were re-solved against the locked v2 sheet. The
ports live in `proto/ports/scout_batch_45/`; the runnable forms in `today/`
were checked against the originals with `check.py` (see the README there for
what is and is not covered).

## Dual LOC

Lines that are neither blank nor a pure comment, from `loc.py`. "Stripped"
drops the contest template (includes, debug macros, aliases, stream setup).

```
problem      raw stripped   v2 today
shoi15p2     166      138   31    70
shoi16p1     108       80   24    57
shoi16p2     131      103   33    54
shoi16p5     174      146   15    24
ucrpc21h     124       94   19    51
iati21a12     96       68   33    41
iati21a23    148      120   54    53
neerc10k     104       76   20    38
rpeng2       107       79   18    37
noimps22p5   162      134   43    54
total       1320     1038  290   479
```

The v2 ports are 78% shorter than the raw originals and 72% shorter than the
template-stripped ones. The forms that build today are 64% and 54% shorter.

The stripped column still contains data structures the author wrote out for
the problem, which a perfect C++ library would also remove: about 85 lines of
`mint` plus a generic segment tree in `shoi16p5`, and about 80 lines of lazy
tree plumbing in `shoi15p2`. Taking those out, the library-stripped total is
roughly 870 lines and the v2 reduction is about 67%.

The one port that did not shrink is `iati21a23`: it is a constructive
simulation whose content is all case analysis, and the v2 file also carries a
14-line `@checker`. Without the checker it is 40 lines against 120.

## Where v2 read well

`shoi16p1` is the clearest win. The original threads a `memo[u][d + maxd]`
table, erases parents out of adjacency lists and repeats the same child loop
five times. With `Rooted`, `@memo` over a signed range and generator sums, the
recurrence is the program:

```cp
@memo
fn need(u: V, d: -k..=k) -> Int:
    let kids = tree.children(u)
    var best = inf
    if d < k: best = need(u, d + 1)
    if d == k:
        best min= cost[u] + sum(need(v, -k) for v in kids)
    ...
```

`shoi16p5` collapses to a named pair and one `SegTree` line, because `Mod<P>`
and the `(id, op, init)` constructor remove both hand-written classes:

```cp
type Affine = (mul: Z, add: Z)
var chain = SegTree<I, Affine>((1, 0), (f, g) => (f.mul * g.mul, f.mul * g.add + f.add), [I] i => leaf(i))
```

`ucrpc21h` shrinks from a 30-line Dijkstra plus a hand-rolled rolling 4D
table to `dijkstra(...)` and an in-place 3D knapsack over a product domain.

## v2 friction

### 1. `max_right` does not say what it returns

`shoi15p2` fills holes left to right until the transplanted tissue runs out,
which is a descent for "first position where the hole count reaches `moved`".
The sheet lists `.max_right(from, ok)` with no return type. While writing

```cp
let end = brain.max_right(to_l, run => run.holes < moved)
brain.apply(to_l..=min(end, to_r), false)
```

I had to guess whether `end` is the last index where `ok` still holds, or the
first where it fails, and what comes back when `ok` holds to the end of the
domain. In a 1-based inclusive domain these differ by one and the off-by-one
is silent. Proposal: state on the sheet that it returns `?I`, the first index
at which `ok` fails (`none` if it never does), so the line above becomes
`to_l..=(end ?? to_r)`. It also has no upper bound argument, so a search
limited to `to_l..=to_r` needs the `min` afterwards.

### 2. A custom `Action` needs the segment length and has nowhere to get it

Range-assign on a node that stores counts has to know how long the segment
is. The original passes `(l, r)` into `from_lazy`. The sheet's
`Action(id, compose, apply)` gives `apply` only the tag and the node, so the
port recovers the length from the node itself:

```cp
(tag, run) => if let hole = tag: solid(hole, run.holes + run.cells) else: run
```

That works here only because `holes + cells` happens to be the length. For a
node that does not carry its size (range-assign with max, say, is fine, but
range-add with sum is not) the user has to add a `len` field by hand.
Proposal: let `apply` optionally take a third `len: Int` argument.

### 3. Small-to-large over containers with a lazy offset has no helper

`iati21a12` merges heaps up a tree while adding the edge weight to every
stored value. In v2 this is a struct wrapping a `MaxHeap` and a `lift`, plus
the standard swap-and-drain:

```cp
pile[u].lift += w
if pile[p].lots.len < pile[u].lots.len: swap(mut pile[p], mut pile[u])
while let (value, count) = pile[u].pop(): pile[p].push(value, count)
```

Three lines is acceptable, but small-to-large is a stock idiom, and the
"offset" wrapper has to be written out by hand. A
`merge_into(mut big, mut small)` free function on any container with
`.len`/`.push`/`.pop` would make the first two lines one, and an
`Offset<Heap>` wrapper in the local library would remove the struct.

### 4. `match` arms that both update state and yield a value

`noimps22p5` picks which element leaves the bag and what the level costs,
per cost mode. The natural shape is a `match (x, y)` whose arms add to `cost`
and evaluate to the element. The sheet does not say whether a multi-line arm
may end in a value expression, so the port had to assume it:

```cp
let gone = match (x, y):
    (0, _):
        cost += held - most
        most
    ...
```

If block arms cannot yield, this becomes a `var gone` assigned in every arm,
which is the "easy to write wrong" form. Worth one line on the sheet.

### 5. Wrapping and clamping into a domain

`rpeng2` walks a ring and `iati21a23` moves a target that stops at the walls.
I wrote `I.wrap(i + left + 1)` and `Cell.clamp(j + step)`; neither is on the
sheet. Today's spelling is `(i + left + 1) % n` on a 0-based domain (and an
off-by-one on a 1-based one) and `min(n, max(1, j + step))`. Both are common
enough in the corpus (circular DP, grid walks) to earn domain methods.

### 6. Keys made of points

`shoi16p2` identifies a square by its four corners. `HashMap<List<Pt>, Int>`
with `sorted(sq)` as the key is short, but it relies on `Pt` being `Ord` and
`List<Pt>` being `Hash`, and neither is stated. A fixed-size `[4]Pt` key
would be the tighter type; the sheet has no literal for an array from four
values.

### 7. Early exit at top level

`ucrpc21h` prints `Impossible!` and stops. The port uses a bare top-level
`return`. If that is not legal the whole rest of the program moves under an
`else:`, which is what the `today/` form does and costs one indent level on
20 lines.

## Prototype gaps hit by the runnable forms

These are not v2 design issues, but each one made a `today/` file longer or
needed a workaround.

A struct or tuple with `Mod<P>` fields does not build: the emitted C++
derives `operator<` for the record and `Mod` has none. The sheet says such
records compile without `Ord`. `shoi16p5` therefore reduces by hand:

```cp
struct Affine:
    mul: Int
    add: Int
... (f, g) => Affine(f.mul * g.mul % MOD, (f.mul * g.add + f.add) % MOD)
```

There is no `LazySegTree` with a custom action and no `max_right`, so
`shoi15p2` carries a 40-line hand-written lazy tree. There is no `SortedBag`
or `MaxHeap`: `noimps22p5` uses a `Set<(Int, Int)>` of (value, serial) pairs
and `iati21a12` negates keys in a `MinHeap`. Method names differ from the
sheet (`Set.insert`/`.min()`/`.max()` instead of `.add`/`.first`/`.last`,
`MinHeap.len()` instead of `.len`, `SegTree.all` instead of `.fold(I)`).
`for c in s` over a `Str` is rejected, `read()` cannot be an argument
(`xs.push(read())`), and `a + x` fails when `a: [V](0..=100)` because the
element type becomes a one-field record instead of a ranged `Int`.

Two prototype behaviours were helpful and should be kept. The range checker
refused `shots.len % (s + c)` because `s + c` may be 0, which is a real
division by zero in the accepted C++ for `s = c = 0`; writing
`let period: 1..=2000 = s + c` documents the precondition. And the alias
check rejected `paint(2 * o, l, mid, tag[o])` where `paint` writes `tag`,
with a message that named the fix.

## Appendix: remaining unported prefixes

Counted from `corpus/submissions/info.json` after this batch: problems with
an accepted C++ submission whose code is not the basename of a file under
`proto/ports/` and whose submission id is not cited there. 273 problems
remain. The prefix is the leading letters of the problem code.

Claimed by an active scout when this batch ran (do not re-partition without
checking): `sac` 26, `dmopc` 18, `cco` 18, `mccc` 15, `ccc` 13, `egoi` 13,
`etsr` 12, `yac` 10, `aac` 6, `dp` 5, `nccc` 5, `btoi` 5, `dmpg` 3, `apio` 3,
`wac` 2, `ceoi` 2, `oly` 2.

Unclaimed:

```
ioi      33   mostly graders/interactive (ioi13+), several output-only
noi      30   many 5-25 KB solutions; noi14p3, noi16p5, noi05p5 are very large
noip     22   older ones (noip99-noip09) are short simulations
acc       4   trivial
bsspc     4   bsspc21s4 is a hard sweep with a custom lazy node
cmt       3   cmt21r1lbc, cmt21r2zjr, cmt21r3cxr: sqrt/block structures, fast IO
ucrpc     3   ucrpc21a, b, c: short
iati      1   iati21a13 (7.6 KB)
noiwc     2   noiwc22p3 is interactive
wossoly   2   short
factorial 2   factorial2 is a 48 KB table
noimps    1   noimps22p2 (12 KB)
ctsc      1   ctsc04p4, tree knapsack with convex merges
coci      1   coci08c1p1, short
one each: tropical, miraclesort, fastbit, quine, helloworld, aplusb (not worth porting)
```

`shoi`, `neerc` and `rpeng` are now fully ported. The only large unclaimed
pools left are `ioi`, `noi` and `noip`; a clean split for the next scouts is
`ioi` / `noi02-noi10` / `noi14-noi23` / `noip`, with the small leftovers
(`bsspc`, `cmt`, `ctsc`, `iati`, `noimps`, `noiwc`, `ucrpc`) as one batch.

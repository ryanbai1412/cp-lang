# Friction backlog, batch 20: statement-first DP, greedy, combinatorics and math

A Stage-1 scout over ten hard DMOJ problems, each solved in CP Lang from the
problem statement before the accepted C++ in `corpus/submissions/` was
opened. The ports live in
[`proto/ports/scout_batch_20/`](../proto/ports/scout_batch_20/): `btoi19p2`,
`btoi19p4`, `cco17p4`, `ccc19s4`, `cco23p1`, `btoi17p5`, `noi02p4`,
`ceoi18p5`, `ioi19p1` and `ioi09p8`.

Three more problems were solved the same way and then dropped before the
push, because sibling batches had landed ports of them in the meantime:
`apio07p2` (APIO 2007 Backup), `ioi07p4` (IOI 2007 Miners) and `dpz` (Frog 3).
Their drafts are not in the repository. Several findings below were made on
those drafts and say so; the snippets quoted from them are the code that was
built and run.

Two limits on "statement-first" should be read before the findings. The
`dmoj.ca` problem pages sit behind a browser challenge, so the statement used
was the DMOJ API metadata (name, limits, types) plus the well-known contest
task, recalled rather than fetched. And the exact input format of each
problem was confirmed from the C++ after the port was written; no port needed
its algorithm changed at that point.

All ten ports build with `proto/cp.py build` (first at commit `f5030cd`,
again after rebasing onto `900c171`) and
`g++ -O2 -std=c++17`, and agree with their originals on 300 small random
inputs each and on one largest-size input
(`python3 proto/ports/scout_batch_20/check.py`). The one exception is
deliberate and is item `W1`. Every error message and every number quoted
below was observed in this batch.

Items are numbered `W1` through `W16`. Earlier items that overlap are cited
and only the new evidence is added.

## Suggested order for a Stage-2 sprint

1. `W5`, `W6`, `W7`: three prototype bugs, two of which produce a wrong
   answer or a hang with no diagnostic.
2. `W9`: `std.*` imports in `proto/cpc`. Three of ten ports wanted one.
3. `W3` and `W8`: typed "increasing", and generators that derive the input
   from a hidden witness. Seven of the thirteen problems solved need one.
4. `W13`: nested recursive functions as the documented home of mutable
   state, with a warning when a `@brute` pair reads a global `var`.
5. `W4`, `W10`, `W11`, `W12`, then the rest.

## (a) Where the statement gave a different design than the C++

### W1. Plus Minus: the port is right where the accepted original is wrong

Every valid grid has all rows alternating or all columns alternating, and
has both exactly when it is a checkerboard. Writing that down from the
statement gives three counts with the same shape, so the port has one
function and calls it three times:

```cp
let by_row = alternating(n, [(f.row, f.plus != odd(f.col)) for f in cells])
let by_col = alternating(m, [(f.col, f.plus != odd(f.row)) for f in cells])
// Checkerboards are counted on both sides: all cells share one "corner sign".
let boards = alternating(1, [(0, f.plus != odd(f.row + f.col)) for f in cells])
print(by_row + by_col - boards)
```

The original (`4456190.cpp`, accepted with 100/100) computes the first two
counts and then guesses the overlap: subtract 1 if both counts are non-zero
and a cell is fixed, 2 if none is. That is wrong when rows and columns can
each alternate but neither checkerboard fits the fixed cells:

```text
5 5 5
- 1 1
+ 2 1
- 5 2
- 5 4
+ 4 2
```

The port prints 6, the original prints 5, and trying all 2^25 grids gives 6.
`check.py` therefore compares this port with an exhaustive count on small
grids, not with the original. A line-by-line port would have carried the bug
over, and a differential test against the original would have called it
correct. This is the strongest argument in the batch for the statement-first
order, and for a `spec` beside every counting port.

### W2. Six problems came out as a smaller algorithm, not a smaller transcription

`btoi19p4`. A chef adds variety to at most `n` meals, so a set of chefs
works exactly when its hours cover the work and `sum(min(hours, n))` reaches
`n * k`. That is one 0/1 knapsack, and optional cells with `max=` and an
optional step on positions carry it:

```cp
var variety: [Paid]?Int = none
variety[Paid.first] = 0
for b in hours:
    for p in Paid.rev:
        if let q = p - b:
            if let v = variety[q]: variety[p] max= v + min(b, n)
```

The original (`4456166.cpp`, 108 lines) splits chefs into those with at
least `n` hours and the rest, runs a counting knapsack over the first group
and a bitset over the second, and joins them with a hand-written search
structure. The two agree on 1,500 random inputs.

`ccc19s4`. The original builds a deque-like structure with `update_all`,
`push_front`, `pop_back` and `query_all` (213 lines with the template). From
the statement, a day in block `d` scores `max(tail[j], head[i])`, and a
maximum of two maxima splits into two suffix maxima over the previous block,
so the port needs three plain arrays and no data structure.

`cco17p4`. The original is a three-dimensional `bool dp[52][n][m]` over
(height level, pillars placed, sum). Placing pillars from tallest to shortest
makes each new pillar independent of the rest, so the port is a one-dimensional
subset sum. The original's table is 650 MB on the stack at `n = 500` and
needs `ulimit -s unlimited` to run at all.

`ceoi18p5`. The original factorises `n`, generates and sorts the divisors,
builds a divisor graph with `lower_bound`, and walks it. The port recurses
over the divisor list with non-decreasing factors and inserts into a
`Set<Int>`; it is 27 lines and five times faster on `735134400` (0.18 s
against 1.03 s).

`ioi19p1`. The original counts inversions per size and then corrects the
total with a second pass over pairs of intervals. The port says what the
statement suggests: pair each shoe with the oldest waiting shoe of its size,
then remove pairs left to right and count the shoes still standing between
them. `[Size]List<Pos>` and `[Pos]?Pos` were accepted as written:

```cp
var waiting: [Size]List<Pos> = []
var served: [Size]Int = 0
var partner: [Pos]?Pos = none
```

`ioi09p8`. Both programs keep two max-trees. For fairs on one day the port
does two monotone sweeps over the day's fairs; the original runs four
prefix-sum passes with `(u + d)` terms. The port's version is the sentence
"within a day the route is monotone" turned into two loops.

On the dropped drafts the same thing happened twice more: `apio07p2` came
out as the regret greedy on a heap with `[Gap]?Gap` neighbours, against a
divide and conquer with min-plus convolution in the original, and `dpz` as a
Li Chao tree over the stone positions, against a pasted line container that
computes intersections with division.

The other three (`btoi19p2`, `cco23p1`, `noi02p4`) reached the same
algorithm as the original.

## (b) Problem constraints the generator cannot be given as a type

Every item here is something `cpl stress` or `fntest` either could not be
told, or could only be told through a `where` that throws inputs away.

### W3. "Strictly increasing" (dropped drafts `dpz`, `apio07p2`; extends `S4`)

Frog 3 reads heights `h1 < h2 < ... < hn`. The only spelling the prototype
accepts is a rejection filter:

```cp
let h: [Stone]1..=1e6 = read() where all(h[i] < h[j] for i in Stone if let j = i + 1)
```

`h.is_increasing` is refused (`[Stone]Int[1..=1000000] has no
.is_increasing`). With the filter, `fntest dpz.cp 3000 5` keeps 477 inputs
and rejects 2,523, and at size 6 it keeps 44 of 300. The acceptance rate
falls like `1/n!`, so no medium-size input is ever produced. `apio07p2` has
the same promise on office positions; the port sidesteps it by testing the
function on the gap array instead of the positions, which is a
re-parameterisation the author has to think of.

A typed spelling would generate instead of filter. Either of these would do:

```cp
let h: [Stone]1..=1e6 = read() increasing
let h: Increasing<[Stone]1..=1e6> = read()
```

### W4. One input bounded by another (`ccc19s4`, `cco23p1`; draft `apio07p2`)

`k <= n` and `2 * k <= n` are written as `where` and rejected, not generated:
128 of 300 inputs for `ccc19s4`, 164 of 300 for the `apio07p2` draft. The bound cannot
go in the type because `n` is read on the same line of thought:

```cp
let n: 1..=1e6 = read()
let k: 1..=1e6 = read() where k <= n      // wanted: let k: 1..=n = read()
```

If `let k: 1..=n = read()` already works for a previously read `n`, the
`where k <= n` idiom in the existing ports that use it should be rewritten and the
generator documentation should say so; it was not tried in this batch.

### W5 is a bug and is listed under (c).

### W8. Inputs that are only valid as the image of a hidden witness (`cco23p1`, `noi02p4`, `btoi17p5`, `ioi19p1`, `ioi09p8`)

`cco23p1` gives the sums of every window of `k` bits. A typed
`[Window]0..=k` array is almost never the window sums of any string, so a
type-driven generator tests only the "answer is 0" path. The original even
asserts that neighbouring sums differ by at most 1 and aborts otherwise. The
useful generator draws the hidden bit string and derives the input:

```cp
gen:
    let bits: [n]0..=1 = random()
    total = [Window] w => sum(bits[i] for i in w.idx..w.idx + k)
```

`noi02p4` promises that an answer of at most 10^6 exists. Two cavemen with
`(cave, pace) = (3, 2)` and `(1, 4)` meet in year 1 for every ring size, so
on that typed-valid input the original prints `-1` and the port never
terminates. `check.py` has to reject such inputs with its own simulation.
This promise is a property of the answer, not of the input, and cannot be a
type; it needs `gen` plus a `where` on a cheap necessary condition. Starting
caves must also be distinct (`distinct(...)` exists for `where`, again as a
filter).

`btoi17p5` needs distinct fixed cells among `k` structs, and the interesting
inputs are the ones consistent with an alternating pattern; uniformly random
signs give 0 almost always. Its sizes are also split: the real limits are
`n, m <= 1e9`, the exhaustive reference needs `n * m <= 12`, and nothing in
the file can say "stress at this scale".

`ioi19p1` needs a row of `2n` non-zero integers in which every size has as
many left shoes as right ones. `[Pos]Int` generates zeros and unbalanced
rows, and no `where` can be satisfied by chance for `n > 2`. The generator
has to draw the sizes and shuffle them with their negatives, which is
`gen` again. This is why the port has no `@brute` pair: `fntest` would call
`swaps` on arrays that are not shoes.

`ioi09p8` needs distinct (day, location) pairs among `n` structs. The port
and the original disagree when two fairs share both: on

```text
4 1 2 4
2 12 10
4 3 15
4 11 29
4 3 6
```

the port prints 33 (a route worth 33 exists: home, 12, 11, 3, home, taking
both fairs at 3) and the original prints 27. Whether the statement allows
such input was not checked against the real page, so `check.py` generates
distinct pairs and this is recorded as a difference, not as a bug in either.

Three more, smaller. `btoi19p2` reads `Char` arrays over the alphabets `.#`
and `NESW?` (and the `ioi07p4` draft over `MFB`); no alphabet type was tried
in this batch, and `check.py` generates those inputs in Python. `btoi19p4`
is typeable, but an input is only interesting when every `need[i] >= k`;
`[Meal]k..=300` was not tried. `ceoi18p5` is fully
typeable (`let n: 1..=1e9`), but a uniform `n` is almost never highly
composite, which is the only hard case. `cco17p4` is the one problem whose
input is fully described by its types: 0 of 300 inputs rejected.

## (c) Bare-core fallback friction, and three bugs found on the way

### W5. `1 << k` is emitted as a 32-bit shift (bug; `btoi19p2`)

The first `btoi19p2` used 32 columns per word:

```cp
const B = 32
let full = (1 << B) - 1
s[i, Word(j.idx / B)] |= 1 << (j.idx % B)
```

The emitted C++ is `(1 << B)` and `(1 << ((ll)(j) % B))` with an `int` 1.
`1 << 32` is undefined, and `1 << 31` is negative and sign-extends into the
upper half of the word. The port passed all 300 small inputs (grids up to
6 x 6) and printed 465,763 reachable cells on a 500 x 500 grid that has
237,539 sea cells. Nothing warned. The port now uses 30 columns per word and
says why. The literal should be emitted as `1LL`, or the shift routed through
the same checked helper that `x << 1` on a variable already uses (`cp_shl`).

### W6. Tuple assignment after `var (a, b) = ...` assigns to a temporary (bug; `noi02p4`; extends `P2`)

```cp
var (r0, r1, x0, x1) = (a, b, 1, 0)
while r1 != 0:
    let q = r0 / r1
    (r0, r1) = (r1, r0 - q * r1)
    (x0, x1) = (x1, x0 - q * x1)
```

builds without a message and emits `const auto& r0 = _d2.f0;` followed by
`Tup3{r0, r1} = Tup3{r1, ...};`, which changes nothing. The extended
Euclid loop never ends. `P2` reported the `const` binding; the new evidence
is that the natural simultaneous update compiles and hangs. The port uses
four `var`s and two temporaries, eight lines for two.

### W7. A `@brute` function with no parameters is run inside `main` (bug; draft `dpz`, `ccc19s4`)

```cp
@brute(cheapest)
fn cheapest_quadratic() -> Int:
    ...
```

`fntest` answers `no @brute(f) or @invariant fns with parameters` (seen on
both), which is fair, but on the `dpz` draft `build` emits the body of the reference as a block at the end of
`main`. The submitted binary then runs the quadratic reference after printing
the answer: 12.1 s on `n = 200000` against 0.04 s once the reference took a
parameter. A parameterless `@brute` should be an error, or better, should
mean "compare on whole program inputs".

### W9. `use std.*` is refused by `proto/cpc` (`cco17p4`, `noi02p4`, `ceoi18p5` — 3 of 10; extends `B9`, `H1`, `P7`)

The statement-first drafts reached for the library three times:

```text
error: line 9: std.bitset.Bitset is library code the prototype compiler has no stand-in for yet (it runs on proto/cpi)
error: line 8: std.math.ext_gcd is library code the prototype compiler has no stand-in for yet (it runs on proto/cpi)
error: line 6: std.math.divisors is library code the prototype compiler has no stand-in for yet (it runs on proto/cpi)
```

Moving to `proto/cpi` (`cp.py lib-run`) was tried first and each draft failed
on a different dialect gap: `v.idx` on a position is `v.to_idx` there,
`[n]Caveman` is "not an index type", and `Set<Int>()` "has no method
insert". So the three ports inline what they need. The cost is 12 lines for
`divisors`, 13 for `ext_gcd`, and for `cco17p4` a change of data structure:

```cp
// wanted
for j in 1..i: next.union(reach.shifted(tall[j] - tall[i]))
// written
for v in 0..=most:
    if reach[Volume(v)]: next[Volume(v + rise)] = true
```

That is a byte per volume instead of a bit, and the port has to track the
largest reachable volume by hand to keep the loop short.

### W10. `Bitset` shifts one way only (`btoi19p2`)

Even on `proto/cpi`, `std.bitset` could not express Nautilus. `shifted(k)`
moves positions up; a move west needs them moved down, and there is no
`shifted(-k)` or `shifted_down`. The port keeps `[Row, Word]Int` and writes
the carries itself:

```cp
if sig == 'E' or sig == '?':
    x |= (at[i, w] << 1) & full
    if let p = w.prev: x |= at[i, p] >> (B - 1)
if sig == 'W' or sig == '?':
    x |= at[i, w] >> 1
    if let q = w.next: x |= (at[i, q] & 1) << (B - 1)
```

`w.prev` and `w.next` as optionals did make the word carries pleasant, and
`i.next` / `i.prev` did the same for the north and south moves. Wanted:
`Bitset.shifted` with a signed count, and a two-dimensional `Bitset<Row, Col>`
whose row step is an `i.next` away.

### W11. Small missing spellings in `proto/cpc`

Each of these cost one failed build:

- `popcount(x)`: `unknown name popcount`. `stdlib/bitset.cp` uses it, the
  prototype compiler does not have it. `btoi19p2` writes a seven-line loop.
- `Z(2).pow(e)`: `Mod<1000000007> has no method pow`. `Z(2) ** e` works.
- `sorted(xs).rev`: `List<Int> has no .rev`, and the same for a `[Pillar]Int`
  array. `cco17p4` builds the descending list with an index comprehension.
- `sorted(found)` for a `Set<Int>`: `sorted needs a list or 1-D array`.
  `[s for s in found]` works because the set iterates in order.
- `some(x)`: `unknown name some`. Returning the bare value from a `-> ?Int`
  function works.
- `let cells: [k]Fixed = [k] _ => read_fixed()`: `table dimension must be a
  domain or range, got Int[0..=100000]`, although `let pos: [n]Int = read()`
  is accepted. `btoi17p5` reads into a `List` with `push`.
- `max(...)!` over a comprehension is required in some places and refused in
  others (`! on a non-optional Int`), depending on whether the compiler can
  see that the range is non-empty. `min(cost[i] + ... for i in Stone.first..j)!`
  needs the `!`; `max(order[j] for j in 0..=i)!` must not have it. The rule is
  sound but invisible when writing; the error should say "this range is never
  empty, so the result is not optional".

### W12. `choose` over `Mod` worked on the first try

Recorded because it is the opposite of friction. In `cco23p1`,

```cp
use std.math.{choose}
const P = 1000003
type Z = Mod<P>
fn ways(total: [Window]0..=1e6) -> Z:
    ...
    choose(free, need)
```

built and matched the original's 60-line modular combinatorics struct. The
whole port is 27 lines against 174.

## (d) Multi-feature collisions

### W13. Recursion, mutable state and `@brute` (draft `dpz`)

The first `dpz` put the Li Chao tree in a top-level `var`, because `add` and
`lowest` are recursive and top-level functions are the obvious place for
recursion:

```cp
var tree: [0..4 * n]?Line = none
fn add(k: Int, lo: Int, hi: Int, line: Line): ...
fn cheapest(c: 1..=1e12) -> Int: ...
```

The program was correct. `fntest` reported a mismatch (`cheapest: 13`,
`cheapest_quadratic: 14`) because it calls `cheapest` a thousand times per
program input and the tree kept the lines of the previous call. The fix is
to nest `add` and `lowest` inside `cheapest` and make the tree a local, which
the prototype supports, recursion included. Three features meet here:
top-level mutable state, recursive helpers, and a harness that calls a
function more than once. A warning when a `@brute` pair writes a global
`var` would have pointed at the cause directly.

### W14. Typed positions against block, tree and coordinate arithmetic (`ccc19s4`, `ioi09p8`, draft `dpz`)

`ccc19s4` cuts the line into blocks of `k`, and `dpz` walks an implicit
tree. Both compute positions with integer arithmetic and then convert on
every access. `ccc19s4` has 12 such conversions in 47 lines:

```cp
best[Cut(i)] = if lo == 0: head else: max(tailed[Cut(j)], plain[Cut(j)] + head)
```

The domain adds nothing here: `i` is a loop variable over `lo + 1..=hi`,
already known to lie in `Cut`. Iterating a sub-range of the domain
(`for i in Cut(lo + 1)..=Cut(hi)`) would remove most of them, and
`let j = max(i - k, 0)` is the only arithmetic that needs a check. For `dpz`
the wish is the usual one: a `std` Li Chao tree keyed by a domain, so the
port never writes `h[Stone(mid)]`.

`ioi09p8` has the opposite direction. `Loc = 1..=500001` is a domain so the
trees can be keyed by it, but the cost of a move is arithmetic on the
location number, so every use reads `x.label`:

```cp
max(from_above.query(Loc.first, x) - down * x.label, from_below.query(x, Loc.last) + up * x.label)
```

`x.idx` would have been off by one without any message, since the domain
starts at 1. The tree also has no "raise this cell" update, so `stand` reads
the cell back first: `from_above.set(x, max(from_above.query(x, x), v))`.

### W15. The loop-size warning fires on the `@brute` reference

```text
dpz.cp: warning: line 57: this loop nest can run ~4.0e+10 times at the largest declared input
apio07p2.cp: warning: line 64: this loop nest can run ~5.0e+09 times at the largest declared input
```

Both were seen on the dropped drafts, and both lines are inside the slow reference, which is slow on purpose and never
runs on a full-size input. The estimator and `@brute` do not know about each
other, so every stressed port builds with a warning that has to be learned
and ignored.

### W16. Two compilers, one port (`cco17p4`, `btoi17p5`, `cco23p1`)

`proto/cpc` has `Mod`, `choose`, `Map`, `Set`, `MinHeap` and `@brute` with
`fntest`, and refuses `std.bitset` and `std.math`. `proto/cpi` has the
library and, as far as this batch got, different spellings for positions,
arrays sized by an `Int`, and sets. A port that wants a bitset and a
`@brute` reference (`cco17p4`) has no home; it gives up the bitset. This is
the collision behind `W9`, and it will grow with every library module added.

## Originals that are not plain programs

Two things about the corpus side, both needed by any automatic differential
test. `ioi19p1` is graded through a function, so `5661847.cpp` has no
`main`; `check.py` appends one, and has to avoid `long` and `endl` because
the original redefines both as macros. And three originals (`ccc19s4`,
`cco17p4`, `ioi09p8`) keep large tables on the stack and crash at the
default 8 MB limit, `ioi09p8` even on a four-line input; `check.py` runs
every binary under `ulimit -s unlimited`.

## Largest-input timings and constant-factor risks

Measured by `check.py` on one largest-size input per problem, port first,
then original: `btoi17p5` 0.06 s / 0.04 s, `btoi19p2` 0.10 s / 0.09 s,
`btoi19p4` 0.02 s / 0.01 s, `ccc19s4` 0.03 s / 0.12 s, `cco17p4`
0.23 s / 0.74 s, `cco23p1` 0.01 s / 0.12 s, `ceoi18p5` 0.18 s / 1.02 s,
`ioi09p8` 0.28 s / 0.35 s, `ioi19p1` 0.04 s / 0.22 s, `noi02p4`
0.01 s / 0.01 s.

Risks that these numbers do not settle:

- `noi02p4` was timed only on an input whose answer is small, because the
  generator must guarantee that an answer exists. The worst case tries up to
  10^6 ring sizes with 105 pairs and one extended Euclid each, about 10^9
  checked arithmetic steps. That case was not measured.
- `cco17p4` does a byte-per-volume subset sum, about 6 x 10^8 byte steps in
  the worst case (500 pillars, 49 distinct shifts, 25,001 volumes). The
  measured input had random heights, which is not the worst case. A real
  `Bitset` would cut this by the word size.
- `btoi19p2` wastes two bits in every word (30 columns instead of 32, `W5`)
  and allocates a fresh `[Row, Word]Int` per signal: 5,000 allocations of
  68 KB. It still matched the `std::bitset` original's time.
- `ioi09p8` builds two trees over all 500,001 locations whatever `n` is, and
  does four tree queries and two updates per fair plus a `sorted` copy of
  the fairs. Measured at `n = 500000` with random days; one day holding
  every fair was not measured.
- `btoi19p4` walks all of `0..=sum(hours)` for every chef: 300 x 90,000
  optional cells in the worst case, 0.02 s measured.
- `btoi17p5` is slower than its original (0.06 s against 0.04 s): it builds
  three lists of pairs and three maps where the original reuses one vector.

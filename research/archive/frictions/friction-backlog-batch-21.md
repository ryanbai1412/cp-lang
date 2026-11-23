# Friction backlog, batch 21: geometry, sweeps and `Float`

A Stage-1 scout pass over ten submissions: four exact ones (a Manhattan
sweep, two rectangle sweeps, and polygons on rational crossings) and six
that compute with reals (an angle, a geometric estimate, two probability
or expected-value tables, a convex-hull-trick DP and a shortest path).
The ports are in
[`proto/ports/scout_batch_21/`](../proto/ports/scout_batch_21/): ioi07p5,
ccc14s4, ioi08p6, nccc10s4, sac22cc4jp3, cmt21peter2, sdoi17r2p1, noi07p2,
apio23p1 and dpj. Items are numbered `E1`... so they do not collide with
the letters of earlier batches (`F`, `G`, `B`, `K`, `S`, `D`, `H`, `T`,
`N`, `L`, `P`, `R`, `M`, `Q`).

What was checked: all ten ports run on the reference interpreter
(`proto/cpi`) and agree with their originals on small random inputs
(`proto/ports/scout_batch_21/check.py`, on top of commit `4256f36`); the
five that print reals agree to a tolerance. Every "today" snippet is a
line of a port or of a probe that was run on `proto/cpi`, and every quoted
output or message was produced by it. The "wanted" snippets do not run.
Nothing was built with `proto/cpc` (it has no `Float`), nothing was run at
full size, and nothing was timed. The convex-polygon and
non-crossing-segment generators of E9 are designs: no port needed them,
so they were not run.

## The minimal `Float` design, in one place

Six ports were enough to settle most of decision 1.5. The proposal:

1. One type, `Float`, an IEEE double. No `long double`: two of the six
   originals (sac22cc4jp3, dpj) use it, and both ports agree with them
   on doubles.
2. A literal is a `Float` only with a `.`: `1e18` stays an `Int`. Mixed
   arithmetic `Int op Float` widens; mixed `min`/`max` is an error (E2).
3. `Int(x)` is refused for a `Float`; say `floor(x)`, `ceil(x)`,
   `round(x)` or `trunc(x)` (E2).
4. Printing takes the digits: `"{x:.6}"` and `fixed(x, 6)`. Bare
   `print(x)` of a `Float` is a compile error (E1).
5. `==` and `!=` on `Float` are a lint that points at `close(a, b, eps)`
   and `sign(x, eps)`, both userland, with `eps` always written (E3).
6. No NaN and no infinity in checked runs: `0.0 / 0.0`, `inf - inf`,
   `sqrt(-1.0)` trap where they are made. "No value" is `?Float`, never
   `1e18` or `-1` (E4).
7. `Float` is ordered by `<` like `Int`, which rule 6 makes a total order
   in checked runs; `sort`, `min`, `Heap` and `dijkstra` need nothing
   special (E4).
8. `sqrt`, `sin`, `cos`, `atan2`, `exp`, `ln`, `hypot`, `PI` are listed
   intrinsics (E5).

## Suggested order for a Stage-2 sprint

1. E1: printing. Every one of the six needs it and it silently does the
   wrong thing today.
2. E2 and E4: the traps that give a wrong answer without a message.
3. E3 and E5: `close`, `sign`, and the missing functions. Userland.
4. E6: points over any number, and crossings without division.
5. E7 and E8: the sweep tools (`Cover`, min with add, compression, 2D
   prefix sums) and a Li Chao tree over points.
6. E9: generators for inputs with a geometric promise, and a tolerance
   for `cpl stress`.
7. E10: small things.

## E1. A `Float` cannot be printed the way a judge wants (6 of 6)

*Checked:* the format digits are accepted and ignored, both ways the plan
spells them:

```cp
let x = 3.14159
print("{x:.2}")             // prints 3.14159
print(x, digits: 2)         // prints 3.14159
print(1e-7 * 1.0)           // prints 1e-07
print(1e16 * 1.0)           // prints 1e+16
```

A judge that reads `1e-07` as wrong, or wants six decimals, fails all of
these, and nothing says so. So every real-valued port imports a userland
`fixed` from `float.cp`, 16 lines that build the digits by hand:

```cp
// today
use real.float.{fixed}
print(fixed(steps[Count(dishes[Pieces(1)]), Count(dishes[Pieces(2)]), Count(dishes[Pieces(3)])], 9))

// wanted
print("{steps[ones, twos, threes]:.9}")
```

The exact minimal design, from what the six needed:

- `{x:.d}` in a string and `fixed(x, d)` as a function, both giving
  exactly `d` decimals, round half away from zero, never an exponent.
- Never `-0.00`: `fixed(-0.004, 2)` is `0.00`. Judges that compare text
  reject the minus sign; `float.cp` tests this case.
- Bare `print(x)` of a `Float` is a compile error naming `{x:.6}`. A
  default is the trap: C++'s six significant digits is the classic wrong
  answer, and cmt21peter2's original prints two decimals where the
  statement's tolerance would have taken six.
- No width, padding or scientific form. None of the six needs one.

cmt21peter2 shows why `check.py` cannot compare text: the original and the
port compute the same sum in a different order, and near a rounding
boundary `71.724999` and `71.725001` print as `71.72` and `71.73`. That is
E9's tolerance.

## E2. `Int` and `Float` mix silently, in four different ways

*Checked on `proto/cpi`:*

```cp
let big = 1e18              // an Int: prints 1000000000000000000
print(1 / 3)                // 0
print(min(2, 2.5))          // 2    (an Int)
print(max(1.0, 3))          // 3    (an Int)
print(Int(-2.7))            // -3   (C++'s (long)(-2.7) is -2)
var total: [P]Float = 0
total[p] += 1
print(total[p])             // 1    (an Int in a Float array)
```

The first is fine and worth keeping: `1e9` as a bound is everywhere. The
rest are why the ports spell every real zero as `0.0` and every real
start value with a point (`var chance: [Total]Float = 0.0`, `var (s, c,
term) = (0.0, 0.0, 1.0)`): an `Int` that slips into a `Float` array stays
an `Int` and prints without decimals, and the annotation does not stop
it. `Int(x)` floors where C++ truncates, so a line-by-line port of
`long(a * 1000)` changes answers for negative values.

```cp
// today: the reader has to know which of these are Int
var (s, c, term) = (0.0, 0.0, 1.0)
let g = gap(Float(aim), atan2(y, x) * 180 / PI)

// wanted: the annotation decides, and conversions say how they round
var total: [P]Float = 0             // 0 is a Float here, because the type says so
let cell = floor(x * 1000)          // Int(x) on a Float: "say floor, round or trunc"
min(turn, 360 - turn)               // Float, because one side is
```

Minimal rule: a literal takes the type it is checked against; `Int op
Float` widens to `Float` (so `term * t / (k + 1)` stays as written);
`min`, `max` and comparisons widen the same way; `Float` to `Int` is only
ever `floor`, `ceil`, `round` or `trunc`.

## E3. Tolerances: three of the originals carry one, one port keeps it

The originals:

```cpp
if (d >= dist[u] - 1e-9) return;            // apio23p1: Dijkstra relaxation
sa[arra[i] > eps].push_back(i);             // sdoi17r2p1: skip tiny products, eps = 1e-5
k += eps; ... gy(i - l - eps)               // noip02p3 (not ported): nudge the interval ends
```

Rethinking removed the first two. apio23p1 uses the library's `dijkstra`,
whose `nd < dist[v]` is right without a tolerance: a tie only costs a
repeated push. sdoi17r2p1's cut-off exists to make polynomial powers fast
enough, and it makes the answers slightly wrong; the window over prefix
sums has no threshold. That is the useful finding: **most `eps` in the
corpus is a patch, not part of the algorithm**, so the language should
make a tolerance visible and deliberate, not convenient.

```cp
// wanted, all userland (float.cp has these two, with tests)
pub fn sign(x: Float, eps: Float) -> Int = if x > eps: 1 elif x < -eps: -1 else: 0
pub fn close(a: Float, b: Float, eps: Float) -> Bool = abs(a - b) <= eps
```

- `eps` is always an argument. A global default hides the one number that
  decides whether a geometry solution passes.
- No `~=` operator and no `Float` type with a built-in tolerance: such an
  equality is not transitive, so it breaks `sort`, `Map` keys and
  de-duplication in ways that depend on the input.
- `==` on two `Float`s is a lint, with `close` in the message. *Checked:*
  `0.1 + 0.2 == 0.3` is `false` without a word today, and a `Map<Float,
  Int>` keyed by `0.1 + 0.2` answers `none` for `0.3`.
- The lint needs one exemption: comparing with a literal zero that was
  stored, not computed (`if cur == 0.0: continue`).

noi07p2 shows the better habit. It needs each day's place among the
sorted price ratios, and looking a ratio up by value would compare
computed reals for equality. The port sorts the days and stores each
day's rank, so no real is ever compared with `==`:

```cp
let by_ratio = sorted(Day, by: d => a[d] / b[d])
for d in by_ratio:
    rank[d] = ratio.len
    ratio.push(a[d] / b[d])
```

sac22cc4jp3 is the case with no good answer: the first point at the
smallest gap wins, the gaps come out of `atan2`, and two points in the
same direction can differ in the last bit. The port compares as computed,
like the original, and `check.py` only generates inputs without ties.
An exact version would compare cross products, but the aim is a whole
number of degrees, whose direction vector is not rational.

## E4. NaN, infinity and "no value" (2 of 6 carry a sentinel)

*Checked on `proto/cpi`:*

```cp
print(0.0 / 0.0)                    // error: division by zero
print(1.0 / 0.0)                    // error: division by zero
let nan = Float.MAX - Float.MAX     // Float.MAX is inf; this is nan, silently
print(nan == nan)                   // false
print(nan < 1.0)                    // false
print(Int(nan))                     // a Python traceback, not a CP Lang error
```

Dividing by zero already stops the run with the line, which is the right
call: in a contest a NaN is always a bug, and it surfaces far from where
it was made (a sort that comes out unsorted, a `max` that depends on the
order of its arguments). The rule should be finished:

- In checked runs every operation that would make a NaN or an infinity
  traps where it happens: `inf - inf`, `0.0 * inf`, `sqrt` of a negative,
  overflow to infinity. The judge build does not check.
- With no NaN, `<` on `Float` is a total order, so `sort`, `min`,
  `Heap<(Float, V)>` and `Order<Float>` need no special case and no
  `total_cmp`. apio23p1 runs the generic `dijkstra` on `Float` unchanged.
- `Float.MAX` should be the largest finite value, not infinity, or be
  dropped: both originals that want "nothing yet" have a better spelling.

That spelling is `?Float`. The originals use `1e18` for "unreachable"
(apio23p1, tested as `dist[i] < 1e18`) and `-1` for "not computed" (dpj's
memo, which is why its table is filled with a loop before use):

```cp
// apio23p1 today: runs, and reads better than the original
let reach = times([(C(0), 0.0)])
if reach[C(h)] == none:
    print(-1)
...
let t = best[c] else: continue
```

This already works on the interpreter and needed nothing new. What is
missing is the layout promise: `?Float` should be 8 bytes, with one NaN
bit pattern as `none` (the plan's niche-packed `?V`), which is only sound
because of the rule above that arithmetic never produces a NaN.

## E5. The functions that are not there (3 of 6)

*Checked:* `sqrt`, `atan2`, `abs` and `**` exist. `sin`, `cos`, `exp`,
`floor`, `round`, `hypot` and `pow` are each "unknown name", `Float.PI`
is "Float has no field or method `PI`", and `7.5 % 2.0` is "no `%` for
7.5 and 2.0". cmt21peter2 needs sine and cosine of 30 angles, so
`float.cp` has a 16-line `sin_cos` (halve the angle, sum a series, double
back), and sac22cc4jp3 imports its own `PI`:

```cp
// today
use real.float.{PI, sin_cos, fixed}
let dir = sin_cos(PI * k / K)
let shadow = [P] p => at[p].x * dir.cos + at[p].y * dir.sin

// wanted: nothing to import
let shadow = [P] p => dot(at[p], unit(PI * k / K))
```

The list for the intrinsics layer: `sqrt`, `sin`, `cos`, `tan`, `atan2`,
`exp`, `ln`, `pow`, `hypot`, `floor`, `ceil`, `round`, `trunc`, and the
constant `PI`. Each is one C++ call. Nothing else was needed by the six,
and `%` on reals was not needed at all: sac22cc4jp3 brings an angle into
`0..360` with two `while` loops, which reads fine.

## E6. Points and lines in userland

`stdlib/geometry.cp` runs on the interpreter today (*checked:* `use
std.geometry.{Pt, cross, ByAngle}`, a polar sort of four points, `cross`
and `Pt(3, 4).len`). Its `Pt`, `dot`, `cross`, `turn`, `ByXY`, `ByAngle`,
`on_segment`, `segments_meet`, `area2`, `hull` and `locate` are the right
set, and exact. What this batch adds is three gaps around it.

**Points are `Int` only.** cmt21peter2 has real points and cannot use
`Pt`, so it keeps `(x: Float, y: Float)` and writes the dot product out.
The minimal change is one generic struct, with the exact helpers staying
`Int`-only so that nobody takes the sign of a real cross product by
accident:

```cp
// wanted
pub struct Pt<T = Int>(x: T, y: T)                  // Pt is Pt<Int>, as today
pub fn dot<T>(a: Pt<T>, b: Pt<T>) -> T
pub fn cross<T>(a: Pt<T>, b: Pt<T>) -> T
pub fn turn(a: Pt, b: Pt, c: Pt) -> Int             // Int only: exact
pub fn turn_eps(a: Pt<Float>, b: Pt<Float>, c: Pt<Float>, eps: Float) -> Int
pub fn unit(angle: Float) -> Pt<Float>              // (cos, sin)
```

**Crossings of lines are fractions.** nccc10s4's original carries a
`Fract` class of about 70 lines with a gcd in every operation, and a `Line` with a
`vert` flag and two cases in `inter`. The port keeps a crossing as a
whole-number triple and never divides:

```cp
// today, in the port: this is the helper std.geometry should have
struct Line(a: Int, b: Int, c: Int)         // a x + b y + c = 0
struct Hom(x: Int, y: Int, w: Int)          // the point (x / w, y / w), w > 0, reduced
fn crossing(p: Line, q: Line) -> ?Hom:
    let w = p.a * q.b - q.a * p.b
    if w == 0: return none
    ...
const LeftToRight = Order<Hom>(lt: (p, q) => if p.x * q.w != q.x * p.w: p.x * q.w < q.x * p.w else: p.y * q.w < q.y * p.w)
```

Vertical lines are not a case, equal points are equal triples (so they
work as `Map` keys, which the port relies on), and order is a comparison
of products. The cost is size: with coordinates up to C the products
reach about 8 C^4, so the checked build's overflow trap matters here and
the docs should give the bound. batch 16's ccc00s5 wrote its own `Frac`
for the same reason; one `Line`, `Hom`, `crossing` and `LeftToRight` in
`std.geometry` serves both.

**The 45 degree turn.** ioi07p5 turns a Manhattan ball into a box with
`(u: x + y, v: x - y)`, on the grid and in the cube, and in the cube
shifts `v` by `m` by hand to keep it a valid index. A domain with a negative start already removes the
shift on the grid (`domain V = 1 - m..=m - 1` runs today). A named helper
`turn45(p: Pt) -> Pt` would be one line and is not worth a library entry
alone; the range clamp around it is the real noise, see E7.

**Polar order.** `ByAngle` is exact and is the right default. sac22cc4jp3
is not a polar sort: it needs the angle as a number, because the aim is
given in degrees. The docs should say so plainly: sort directions with
`ByAngle`, and reach for `atan2` only when an angle is printed or given.

## E7. Sweeps: four tools written by hand

**Covered length.** ioi08p6 slides a window of columns and asks for the
longest run of rows with no obstacle. The standard tool is a segment tree
that counts the intervals spanning each node and pushes nothing down,
valid because intervals are removed whole. It cannot be written with
`LazySegTree` (its update does not compose as an `Action`), so the port
has it by hand: 36 lines, a third of the file.

```cp
// today: struct Cover in ioi08p6.cp, four lists and a recursive add
var cover = Cover.new(n)
for b in enters[X(right)]: cover.put(b.y0, b.y1)
while cover.longest_free < right - left + 1: ...

// wanted: in std, over a domain, with the two questions sweeps ask
var cover = Cover<Y>()
cover.put(b.y0..=b.y1)
cover.free_run          // longest uncovered run
cover.covered           // how much is covered: area of a union of rectangles
```

**Smallest value under range add.** `std.lazy` has `MaxOpt` and `AddMax`
over `?Int` only, so ioi08p6 defines the mirror image, with a check for
the identity inside `apply`:

```cp
// today
const Cheapest = Monoid<Int>(id: Int.MAX, op: (a, b) => min(a, b))
const Dearer = Action<Int, Int>(id: 0, compose: (f, g) => f + g, apply: (f, x) => if x == Int.MAX: x else: x + f)
var cost = LazySegTree<Y, Int, Int>.from(Cheapest, Dearer, [Y] _ => 0)
best min= cost.fold(Y.first..=Y.last)

// wanted
var cost = AddMin<Y>(0)
best min= cost.all
```

**Compression.** ccc14s4 sorts and de-duplicates the pane edges and
binary-searches each one, 11 lines for `cuts` and `rank`. The compiled
prototype has `compress(ends)` returning a domain; the interpreter does
not, and the port also wants the widths of the strips between cuts:

```cp
// wanted
let (X, x_at) = compress(p.x0 for p in panes, p.x1 for p in panes)
for i in X.gaps: ... x_at[i.r] - x_at[i.l]      // the strip between two cuts
```

**Prefix sums in 2D, and arrays over boundaries.** ioi07p5 (per layer of
the cube) and ccc14s4 both build 2D prefix sums with three `if let ...
prev` lines and query a box with four clamped corners. sdoi17r2p1 wanted
a 1D prefix array indexed by boundaries and got: "`Bounds<Total>` is not
an index type". All three fell back to raw numbers.

```cp
// today (ioi07p5)
let (a, b) = (U(max(u0 - 1, 1)), U(min(u1, 2 * m)))
let (c, e) = (V(max(v0 - 1, 0)), V(min(v1, 2 * m - 1)))
seen[z, b, e] - seen[z, a, e] - seen[z, b, c] + seen[z, a, c]

// wanted
let boxes = prefix2(seen[z])
boxes.sum((u0..=u1).clamp(U), (v0..=v1).clamp(V))
```

`(lo..=hi).clamp(D)`, a range of numbers cut to a domain, would remove
every `max(.., first)` and `min(.., last)` in ioi07p5 and ioi08p6.

## E8. A Li Chao tree for reals (1 of 6)

`examples/local-lib/ds/li_chao.cp` is `Int` only and lives on integer
points `lo..=hi`. noi07p2 has real slopes and asks at real points, so the
port carries its own tree, 23 lines, over the days sorted by price ratio.
The general structure is the one the port wrote: a tree over a sorted
list of query points, generic in the number.

```cp
// wanted
var lines = LiChao<Float>.over(ratio, Greater<Float>)       // highest line wins
best max= b[d] * (lines.at(rank[d]) ?? 0.0)
lines.add(Line(coupons_b * rate[d], coupons_b))
```

Asking by rank, not by value, keeps E3's habit: no real is looked up by
equality.

## E9. Inputs with a geometric promise, and answers with a tolerance

Every generator in `check.py` is hand-written Python, because none of the
promises can be said by a field type. What each one needed, and what
`cpl stress` should offer:

**A tolerance.** Five of the ten ports print reals and cannot be compared
as text (E1). One argument is enough:

```cp
// wanted
$ cpl stress dpj.cp original.cpp --close 1e-9      // absolute or relative, per token
```

**General position.** sac22cc4jp3 needs "no two gaps equal", or the
answer depends on the last bit of `atan2`. This one is fine as rejection,
and the existing `where` clause says it:

```cp
let pts: [P]Pt = read() where distinct(gap(aim, p) for p in pts)
```

The opposite matters as much. nccc10s4's generator draws lines through a
7 by 7 lattice on purpose, so that parallel lines, repeated lines and
three lines through one point are common: that is where exact arithmetic
earns its keep. A geometry generator needs both modes, `rand_points(n,
box, general: true)` (no three on a line, no two on a vertical) and a
tiny box for degenerate cases, and the default small box of `cpl stress`
already gives the second.

**Rectangles that do not overlap** (ioi08p6). Rejection almost never
succeeds for more than a few rectangles. The generator is constructive:
cut the field into slabs and cells, and put at most one rectangle inside
each cell.

**Convex polygons** (not needed by a port; design only). Sorting random
points by angle does not give a convex polygon, and taking the hull of
random points gives very few corners. The standard construction draws
edge vectors that sum to zero and sorts them by angle; with `ByAngle`
that is five lines of userland:

```cp
// wanted, in std.geometry
pub fn rand_convex(n: Int, box: Int) -> List<Pt>:
    var steps = zero_sum_vectors(n, box)        // random x steps and y steps, each summing to 0
    sort(mut steps, by: ByAngle)
    scan(steps, (a, b) => a + b)                // running sum: the corners, counter-clockwise
```

Parallel steps give corners on a straight line, so `strict: true` should
merge them.

**Segments that do not cross** (design only). batch 16's ioi07p2 has the
lattice version: only join lattice neighbours. A second construction that
gives long segments is to draw segments one at a time and keep a segment
only if `segments_meet` is false against those kept, which is quadratic
and fine at stress sizes.

All of these are functions returning a whole input, so they fit the
`read(gen: f)` hook of batch 16's Q1. The new request is that `rand_convex`,
`rand_points(general:)`, `rand_disjoint_rects` and `rand_segments(crossing:
false)` ship in `std.geometry`, next to the helpers they exercise.

## E10. Small things

- `read()` is only understood in `let x: T = read()`. `xs.push(read())`
  and `a[d] = read()` are "unknown name `read`", so ioi07p5 and ccc14s4
  read each number into a name first. Tuples do work: `let (u, v, time):
  (Int, Int, Float) = read()`.
- A named tuple cannot be built by position: `at.push((x + y, x - y))`
  into a `List<(u: Int, v: Int)>` runs, and fails later with "(7, 3) has
  no field `u`". It should be accepted, or refused at the `push`.
- `[3]Float` and `[1..=3]Int` are "not an index type" on the interpreter;
  dpj declares `domain Pieces = 1..=3` to count three kinds of dish.
- No `enumerate` on a list (cmt21peter2 counts by hand) and no `fill` on
  a `List` (nccc10s4 clears with a loop).
- `let x: Int = read()` on the token `2.5` is a Python traceback.
- A function must be declared above the top-level line that calls it:
  calling `f` one line before `fn f` is "unknown name `f`".
- Originals: apio23p1 is a grader function without `main`, and ioi07p5
  keeps a 240^3 array on the stack and needs an unlimited stack to run at
  all. `check.py` handles both; `cpl stress` should take a `--main` file
  and raise the stack limit for the reference.

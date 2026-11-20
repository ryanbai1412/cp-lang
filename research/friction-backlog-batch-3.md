# Friction backlog, batch 3 (2026-11-20)

Strings, number theory, geometry and one interactive problem. This file
continues [`friction-backlog.md`](friction-backlog.md) (batch 1, items
`F1` to `F21`); its own items are `G1` to `G25`. An `F` item is repeated
here only where this batch hit it again.

Ten problems, sketched in
[`proto/ports/scout_batch_3/`](../proto/ports/scout_batch_3/): ccc03s4,
noip99p2, coci06c1p6 (strings), modinv, aac4p3, ccc18s4, dpr (number
theory and counting), noip01p4, cco04p4 (geometry) and cco09p4
(interactive). The sketches use the syntax we want, and each wished line
carries its item number.

What was checked and what was not. Every "today" snippet below is taken
from a file in `proto/ports/scout_batch_3/today/` that builds with
`proto/cp.py` at commit `c217120`; seven of those files also agree with
their original under `cp.py stress` or `interact.py`, and the other two
agree on inputs made by a script (the folder's README says which and why).
Every error message quoted and marked *checked* comes from a small probe
built at the same commit. The ten sketches themselves do not build and
were not run. A count such as "2 of 10" is the number of sketches in this
batch that carry the marker. Corpus counts are a text search of the 710
problems with an accepted C++ submission, over whole files, so they
include the author's template; treat them as upper bounds.

`proto/cpc/` and `crates/` were not changed.

## Suggested order for the Stage-2 sprint

1. G8, G25 and G1. The first reads wrong input without a compile error,
   the second builds and then fails in the C++ compiler, and the third
   rejects a correct program.
2. F1 again: 5 of these 10 sketches import from `std`, after 4 of 10 in
   batch 1. It is the largest single item in both batches.
3. G2, G4, G5: small pieces of the language or the prelude, 2 or 3
   problems each, and common across the corpus.
4. F15: both geometry problems stop at floats, and noip01p4 has no
   buildable form at all.
5. Library pieces, one or two problems each: G12, G13, G11, G7, G22, G21,
   G15, G16.
6. Small ones: G3, G19, G20, G17, G23, G24, G14, G18, G10, G9, G6.

"Simplifies" under an item counts problems of this batch unless it says
otherwise.

## Bugs and false errors

### G8. A grid of a Char domain is read and generated as numbers (bug; coci06c1p6)

*Checked.* The grid is rows of packed `0` and `1`. Declared as it should
be:

```cp
domain Bit = '0'..='1'
let g: [Row, Col]Bit = read()
```

this builds, and on the input `000` it stops at run time with `domain
value out of range`, because each cell is read as an integer label. In
generator mode the same declaration prints character codes (`48 49 49
49`). Declared as `[Row, Col]Char` the packed rows are read correctly,
but the generator prints random letters with spaces between them (`c a c
b`), which the original cannot read. So no declaration gives both a
correct reader and a usable generator, and `cp.py stress` cannot be used
on any packed grid; `today/coci06c1p6.cp` was compared with its original
by a separate script.

Proposal: a 1-D or 2-D array whose cell type is `Char` or a domain of
characters is read as packed words and generated the same way, with the
cells drawn from the domain. The audit already did this for one word
(`[domain S]Letter = read()`). This is the same gap as the second half of
F3. Simplifies 1 problem here and every `.`/`#` grid in the corpus.

### G25. A record with an enum field builds, and the C++ does not (bug; cco04p4)

*Checked.* A move is a length and an axis, `3 X`:

```cp
@tags("X", "Y")
enum Axis:
    X
    Y
let legs: [n](d: -100..=100, axis: Axis) = read()
```

`cp.py build` accepts this, and `g++` then rejects the generated file with
`no match for 'operator<' (operand types are 'const E_Axis' and 'const
E_Axis')`, after some 1,700 lines of template notes: the record gets an
ordering, and the enum has none. Today the two fields are read one at a
time in a loop, and the moves are pushed as they are read:

```cp
for _ in 0..n:
    let d: -100..=100 = read()
    let axis: Axis = read()
```

Proposal: enums get the ordering of their declaration, or a record only
gets `<` when every field has one. Either way the error must come from
`cp.py`, never from `g++`. Simplifies 1 problem here; every "one command
per line" input with a tagged field is this shape.

### G1. `gcd` of two positive numbers "may be 0" (false error; aac4p3)

*Checked.* With `s.a` and `s.b` declared `1..=1e9`:

```cp
let g = gcd(s.a, s.b)
directions.insert((s.a / g, s.b / g))
```

gives `` `g` may be 0 in `s.a / g` ``. The result of `gcd` is typed
`0..=hi` whatever the arguments are. `assert g > 0` does not remove the
error; what does is a statement that can never run:

```cp
if g == 0: continue
```

Proposal: `gcd(a, b)` is at least 1 when either argument's range excludes
0, and `lcm(a, b)` when both do. Separately, `assert e` should give the checker the
same fact that `if not e: return` gives. Simplifies 1 problem here; every
reduced fraction and every "divide by the gcd" step meets it.

## Language and prelude

### G2. The mirror image, as a value (3 markers in 2 of 10: noip99p2, coci06c1p6)

A position already has `i.mirror`, and it is what makes the turned grid in
coci06c1p6 one line. A list and a range do not have it. *Checked:* `d ==
d.rev` gives ``List<Int> has no `.rev` `` (`rev` reverses in place, or
adapts a loop), `span(i, j).rev` gives ``Range<Row> has no `.rev` ``, and
`span(i, j).mirror` gives ``Range<Row> has no `.mirror` ``. Today:

```cp
if all(d[i] == d[d.len - 1 - i] for i in 0..d.len): return k
for i in 0..a.len:
    carry += a[i] + a[a.len - 1 - i]
let there = turned.sum(span(r2.mirror, r.mirror), span(c2.mirror, c.mirror))
```

Wanted:

```cp
if d == d.rev: return k
for (x, y) in zip(a, a.rev):
seen.of(rs, cs) == turned.of(rs.mirror, cs.mirror)
```

Proposal: `xs.rev` in an expression is a reversed view of a list, an
array or a slice, and the in-place form gets its own name (`xs.reverse()`).
`r.mirror` on a range of a domain is the range of the mirrored positions.
The `n - 1 - i` it removes is the most common off-by-one in string code;
`reverse(` appears in the originals of 45 corpus problems.

### G4. Text next to a value, and decimals (3 of 10: noip99p2, noip01p4, cco04p4)

noip99p2 prints `STEP=3`. *Checked:* `print("STEP=" + k)` gives `` `+`
needs Int operands, got Str and Int ``; `print("STEP=", k, sep: "")` gives
`unsupported expression kwarg`; `print("STEP={k}")` builds and prints the
braces as they are. Today it is two statements:

```cp
write("STEP=")
print(k)
```

The two geometry problems print one and two decimals, and `print(x,
digits: 1)` gives the same `unsupported expression kwarg`. Proposal: `{e}`
inside a string literal is replaced by the printed form of `e`, and
`{e:.2}` fixes the decimals; `print(x, digits: 2)` can stay as the short
form for a lone number. `Case #1: ...` output needs the same thing. Float
output appears in 30 corpus problems.

### G5. A set or a list straight from a generator (2 of 10: aac4p3, cco09p4)

*Checked:* `Set(x * 2 for x in a)` gives `unknown name Set`,
`counts(e for p in points)` gives `a bare generator expression needs a
reduction (sum, min, ...) or List(...)`, and the `List(...)` that the
message suggests gives `unknown name List`. A list has the comprehension
`[e for x in xs]`, so `counts([...])` works; a set has nothing:

```cp
var directions: Set<(Int, Int)> = []
for s in slopes:
    directions.insert(...)
pair_up([c for c in Cap], [b for b in Bottle])
```

Wanted: `Set(f(s) for s in slopes)` and `List(Cap)`. Proposal: `List(it)`,
`Set(it)` and `counts(it)` take anything a `for` can loop over (a
generator expression, a range, a domain), and the error message stops
recommending a name that does not exist.

### G19. `match` as an expression (cco04p4)

*Checked:* `let d = match a: ...` gives `expected an expression (at
'match')`. `match` is a statement only, so a value picked by a variant is
assigned inside each arm:

```cp
match axis:
    X => moves.push((dir, 0))
    Y => moves.push((0, dir))
```

Wanted: a `unit(leg)` whose body is `match leg.axis:` with `X => Pt(dir,
0)` and `Y => Pt(0, dir)` as its arms, so the list of moves is one
comprehension.
`if` is already an expression; `match` should be one in the same places.

### G17. `unreachable` (noip01p4)

*Checked:* `unknown name unreachable`. The fourth corner of a rectangle is
found by trying each given corner as the right angle; one of the three
always is, and the function has to end with something. Proposal:
`unreachable` is an expression of any type that stops with the line number
in a local build. The same word would end G1's dead `if g == 0`.

### G18. A record of domains as an index (noip01p4)

An airport is a city and a corner. *Checked:* with `type Airport = (city:
City, corner: Corner)`, `var at: [Airport]Int` gives `array dimension must
be a domain, a range, or a struct with `impl Index`, got (city: City,
corner: Corner)`. `[City, Corner]T` works, but then a single airport is
two loose values, and a table of distances between airports is
`[City, Corner, City, Corner]F64` with four indices at every use. The
original flattens to `4 * i + j` by hand. Proposal: a tuple or record whose
fields are all domains is an `Index` without an `impl` (its `to_idx` is
the product layout that `[A, B]T` already uses), so `[Airport]T` and
`[Airport, Airport]T` work and `for a in Airport` loops over all of them.
This is F4 without the `where`.

### G14. Reading into `Mod` (dpr)

*Checked:* `let step: [V, V]Z = read()` gives `cannot read
Mod<1000000007>`. An `Int` literal is already accepted where a `Mod` is
wanted, so only the input needs a second table:

```cp
let adj: [V, V]0..=1 = read()
var step: [V, V]Z = [V, V] (i, j) => Z(adj[i, j])
```

Proposal: `read()` into `Mod<P>` reads an `Int` and reduces it.

### G9. Numbers past `Int` (2 markers in modinv; decision 1.7 is leaning)

The judge gives x and m below 2^64, and the original reads `uint64_t` and
runs Euclid in `__int128`. *Checked:* `U64` is an unknown type. The
`today/` form declares `1..=1e18`, which is a different problem. This
needs decision 1.7 settled before a sprint; the sketch assumes `U64` can
be read and compared, and `I128(x)` converts. Wide integers appear in the
originals of 45 corpus problems, most of them for hashing or a product
before a `%`, which `mul_mod` and `Mod` already cover.

## Library

### F1 again. Ports cannot use the standard library (5 of 10)

ccc03s4 (`suffix_array`, `lcp_array`), modinv (`ext_gcd`), noip01p4 and
cco04p4 (`Pt`), ccc18s4 (the proposed `quotients`). Nothing new to say
beyond batch 1, except the size of the effect: with `std.strings`,
ccc03s4 is 7 lines against 11 today, and the 11 are an O(n^2 log n) hash
set where the sketch is a suffix array. modinv writes `ext_gcd` out again
although `stdlib/math.cp` has the same function.

### G12. Runs of equal quotients (ccc18s4)

`trees(w)` sums `trees(w / k)` over `k` in `2..=w`. Only about `2 *
sqrt(w)` different quotients occur, and the whole solution is walking
them. Today:

```cp
var k = 2
while k <= w:
    let q = w / k
    let last = w / q
    total += (last - k + 1) * trees(q)
    k = last + 1
```

Wanted:

```cp
sum(ks.len * trees(q) for (q, ks) in quotients(w, 2..=w))
```

Proposal: `quotients(n, range)` in `std.math` yields each quotient with
the maximal range of divisors that give it. The original splits the same
sum at `sqrt(x)` into two loops with different bounds, which is the
version people get wrong. Divisor-sum and Dirichlet-style problems all use
this loop.

### G13. A binary power of anything (dpr)

Today the squaring loop is written out next to the product:

```cp
var e = k
while e > 0:
    if e % 2 == 1: walks = mul(walks, step)
    step = mul(step, step)
    e /= 2
```

Wanted: `let walks = power(step, k, times, one)`. Proposal: `power(x, k,
op, one)` in the prelude for any associative `op`; with a `Monoid` from
`std.algebra` it is `m.power(x, k)`. `Mod.pow` and the `Mat.pow` in
`examples/local-lib/math/matrix.cp` are the same loop written twice more.
That `Mat` also stores `List<List<Mod<P>>>` and indexes by `Int`; a
`[V, V]Z` table, as in the sketch, keeps the vertex domain. Matrix powers
appear in 3 corpus problems, permutation and function powers in more.

### G11. Pairs of equal keys (aac4p3)

Two points fit a slope a / b when `b * y - a * x` agrees, and the answer is the
number of pairs with equal keys. *Checked:* a `Counter` cannot be looped
over (`cannot iterate Counter<Int>`), so `sum(c * (c - 1) / 2 for (_, c)
in counts(...))` is not available, and today it is a map and the
count-before-insert trick:

```cp
var on_line: HashMap<Int, Int> = []
for p in points:
    let line = b * p.y - a * p.x
    pairs += on_line.get(line) ?? 0
    on_line[line] += 1
```

Wanted: `equal_pairs(b * p.y - a * p.x for p in points)`. Proposal: make a
`Counter` iterable as `(key, count)`, and add `equal_pairs(it)` on top of
it. "How many pairs agree on f" is one of the commonest counting steps.

### G10. A reduced fraction (aac4p3)

Slopes 2/4 and 1/2 are the same. Today the reduction is done by hand into
a tuple (the snippet under G1). Wanted: `Set(Frac(s.a, s.b) for s in
slopes)`. Proposal: `Frac(num, den)` in `std.math`, kept reduced with a
positive denominator, with `==`, `<` and hashing. The original's `gcd`
leaves the sign where it was, which only works because every input is
positive; a library type removes that trap.

### G7. A hash of any rectangle (coci06c1p6)

`StrHash` covers one dimension. In two, today's form builds the hash from
`prefix` over weighted cells and compares by cross-multiplying, since
there is no modular inverse at hand:

```cp
fn weight(r: Row, c: Col, ch: Char) -> Int: (ch - '0' + 1) * row_pow[r] % P * col_pow[c] % P
let seen = prefix([Row, Col] (r, c) => weight(r, c, g[r, c]))
...
return here * row_pow[r2.mirror] % P * col_pow[c2.mirror] % P == there * row_pow[r] % P * col_pow[c] % P
```

That plumbing is more than half of the file's 23 lines, and the line above is the kind that is
wrong on the first try. Wanted: `Hash2D(g)` with `h.of(rows, cols)`, built
from any `[R, C]T` or a table lambda. Proposal: write it in `stdlib/` over
`prefix` and `Mod`, with the same interface as `StrHash`.

### G6. A suffix array that knows what it sorts (ccc03s4)

`suffix_array` in `stdlib/strings.cp` takes a `List<T>` and returns
`List<Int>`, and `lcp_array` returns `List<Int>` too. For ccc03s4 that is
enough, since only the sum is used. Any problem that goes on to index the
string with the result gets raw `Int` positions back, which is what
domains exist to prevent. Proposal: for `s: [S]T` return `[Rank]S` with a
fresh `Rank` domain, and `lcp` as `[Rank]Int`; this also fits decision
1.6, where a string is `[domain S]Char`.

### G15. Arithmetic on points in a port (2 of 10: noip01p4, cco04p4)

*Checked:* with `type Pt = (x: Int, y: Int)`, `a + b` gives `` `+` needs
Int operands, got (x: Int, y: Int) and (x: Int, y: Int) ``. `std.geometry`
has `Pt` with `+`, `-`, `dot` and `cross`, which ports cannot import (F1),
so today each step is taken field by field:

```cp
best min= (p.x - q.x) * (p.x - q.x) + (p.y - q.y) * (p.y - q.y)
p = (p.x + da.x, p.y + da.y)
```

Wanted: `(p - q).norm2` and `p + da`, and `q + r - p` for the fourth
corner. Two
additions to `stdlib/geometry.cp` once F1 and F15 land: `norm2` (an
`Int`) and `dist(a, b)` (a float). The original of noip01p4 writes the
right-angle test as `(y1 - y2) * (y1 - y3) == -(x1 - x2) * (x1 - x3)`; in
the sketch it is `dot(q - p, r - p) == 0`.

### G16. All-pairs shortest paths (noip01p4)

The original is the three Floyd-Warshall loops over a `double` table.
Wanted: `let cost = all_pairs(price)` over `[Airport, Airport]F64`.
Proposal: `all_pairs(w)` in `std.graph` for any `[V, V]T` with `+` and
`min`, where a missing edge is `none`. It joins the audit's open "Graph
boilerplate" list.

### G21. `zip`, `cycle`, `take` (2 markers: cco04p4; noip99p2 uses `zip`)

*Checked:* `zip(a, b)` gives `unknown name zip`, though `plan/20-stdlib.md`
lists it among the views. Two craft with lists of different lengths start
again together after `lcm` ticks. Today the two lists are walked with a
shared counter and `%`:

```cp
for t in 0..lcm(a.len, b.len):
    best min= ...
    let da = a[t % a.len]
    let db = b[t % b.len]
    p = (p.x + da.x, p.y + da.y)
    q = (q.x + db.x, q.y + db.y)
```

Wanted:

```cp
let path_a = scan(a.cycle, +, from: start_a)
let path_b = scan(b.cycle, +, from: start_b)
let closest = min((p - q).norm2 for (p, q) in zip(path_a, path_b).take(ticks))
```

Proposal: implement `zip` first, since it is the one two problems here
use; `xs.cycle` and `it.take(k)` are the next two views. They have to be
lazy, since `a.cycle` has no end. The path is F2's `scan` again.

### G22. Comparison results, and a three-way split (2 markers in cco09p4)

The judge answers -1, 0 or 1. An enum with those tags can be declared
today (*checked:* `@tags("-1", "0", "1") enum Order` as the type of an
`ask` builds, and a reply of -1 fed by hand takes the `Less` arm), but each program would declare its own, and
the split is three lists filled in a loop:

```cp
var small: List<Cap> = []
var big: List<Cap> = []
var fits: ?Cap = none
for c in caps:
    let r = fit(c, pivot)
    if r < 0: small.push(c)
    elif r > 0: big.push(c)
    else: fits = c
```

Wanted: `let (small, fits, big) = split(caps, by: c => try(c, pivot))`.
Proposal: a prelude `Order` (`Less`, `Equal`, `Greater`, with `.flip`)
that reads from -1, 0, 1, is what a comparator may return, and is the key
of `split(xs, by:)`, which returns the three lists. With it the whole
solution is 13 lines against 24. Every comparison-based interactive
problem gets the same answers.

### G23. A random element (cco09p4)

*Checked:* `a.pick()` gives ``List<Int> has no method `pick` ``. Today:
`bottles[rand(0, bottles.len - 1)]`, after a separate emptiness test.
Wanted: `let pivot = bottles.pick() else: return`. Proposal: `xs.pick()`
is `?T`, `none` when empty, and `xs.shuffle()` beside it. Randomised
choices appear in 16 corpus originals.

### G24. The single element (cco09p4)

Exactly one cap fits the pivot. Today a `?Cap` is filled in a loop and
unwrapped with `!`. Wanted: `fits.only`, which stops with a message when
the list does not have exactly one element. Small; it also says in the
code that the count is a promise of the problem.

### G3. The value of a digit character (noip99p2)

Base 16 input uses `0` to `9` and `A` to `F`. *Checked:* `if c in
'0'..='9'` gives `a range can't start or end at a Char`. Today:

```cp
fn value(c: Char) -> Int:
    if c <= '9': c - '0' else: c - 'A' + 10
```

which is right only because of where the digits sit in ASCII. Wanted:
`c.digit(base)`. Proposal: `Char.digit(base) -> ?Int` and its inverse
`Char.of_digit(d, base)`; and allow `Char` ranges, which the `domain
Letter = 'a'..='z'` form already has. `isdigit` or `- '0'` appears in the
originals of 36 corpus problems.

### G20. `sign` (cco04p4)

*Checked:* `unknown name sign`. Today: `let dir = if d > 0: 1 else: -1`,
which is wrong for 0 and harmless here only because a leg of length 0
takes no step. Proposal: `sign(x)` in the prelude, typed
`-1..=1`. With G22 it could return an `Order`; that is a choice for the
sprint.

## Seen again from batch 1

- **F15, floats** (2 of 10). noip01p4 has no buildable form, and cco04p4
  stops one line before its answer. *Checked:* `F64` is an unknown type and
  `sqrt` an unknown name. `sqrt` or `hypot` appears in 9 corpus originals
  and float output in 30.
- **F6, windows of a domain** (coci06c1p6). Today `killer(r, c, size)`
  takes a corner and a size and rebuilds both ranges with `if let r2 = r +
  (size - 1) and let c2 = c + (size - 1)`; with `Row.windows(size)` the
  function takes the two ranges and the `false` fall-through goes away.
- **F2, `scan`** (cco04p4). Where a craft is at each tick is the running
  sum of its moves.

## Already good

Things this batch needed that built on the first try: `@memo` on a plain
`Int` argument up to 1e9 (ccc18s4), a recursive function returning a named
record with `ext_gcd(x, m).x % m` needing no sign fix because `%` is
Euclidean (modinv), `[V, V] (i, j) => sum(a[i, m] * b[m, j] for m in V)`
as a whole matrix product and `sum(table)` over a 2-D table (dpr), a
payload-free `@tags` enum read on its own, and one with the tags -1, 0, 1
as the type of an `ask` (cco04p4, cco09p4), reads inside a function that
is called twice (cco04p4), `ask` into a range type and list
comprehensions that call it (cco09p4), `i.mirror` and 2-D `prefix` with
`span` (coci06c1p6), `max(... if any(...))` as an optional with `?? -1`
(coci06c1p6), `StrHash` with `hash.of(span(i, j))` (ccc03s4), `where
gcd(x, m) == 1` rejecting generated inputs (modinv), and a `for` over a
list of tuples with a destructuring pattern.

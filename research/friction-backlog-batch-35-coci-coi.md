# Friction backlog, batch 35: COCI and COI

A scout pass over the `coci*`/`coi*` bucket of the corpus: a DP on a graph
with one cycle (coci21c1p3), a Fourier transform over Z_3^k (coci21c1p4),
disjoint longest increasing subsequences (coci21c1p5), tree diameters in a
forest (coci21c3p2), a two-pointer over row bands (coci21c6p2), a
floating-point sweep (coi07p1) and four easy first and second problems
(coci21c2p1, coci21c2p2, coci21c3p1, coci21c6p1). The ports are in
[`proto/ports/scout_batch_35/`](../proto/ports/scout_batch_35/). Items are
numbered `K1`... so they do not collide with earlier batches.

What was checked, and what was not. Each port exists twice: `<code>.cp` as
the v2 cheat sheet (`docs/one-page-cheat-sheet.md`) spells it, and
`today/<code>.cp` in the forms the prototype takes. Only the `today/` forms
ran: eight built to C++17 by `proto/cp.py build` and compared with the
original on 200 random inputs each, two (coi07p1, coci21c2p1) on the
reference interpreter on 60 inputs each
(`python3 proto/ports/scout_batch_35/check.py`, `main` at `c8e381f`). The
v2 files were not run by anything. Every "today" snippet is in a `today/`
file and every error quoted came from that prototype; "v2" snippets are
what the cheat sheet reads as allowing and are unverified. Timings are
from one machine at `-O2` on hand-made large inputs, one run each. The
DMOJ API returned 403 for every problem, so statements were reconstructed
from the accepted C++; for coi07p1 nothing else was available. Nothing was
run on the Rust compiler, and `crates/` and `proto/cpc/` were not changed.

The brief asked for ten hard problems; the bucket had three (coci21c1p3,
p4, p5). Twelve problems were unported when the batch was picked and the
two left out are easier still. Items K1 to K5 come from the three hard
ports and rest on one port each.

## Line counts

`python3 proto/ports/scout_batch_35/loc.py` counts non-blank, non-comment
lines. "Stripped" removes from the original what a perfect C++ contest
library would supply: the template header and driver everywhere, plus the
cycle finder in coci21c1p3, the buffered I/O and the `cint` number type in
coci21c1p4, and the generic segment tree in coci21c1p5 (ranges in
`<code>.strip`).

```text
problem        v2  today   C++  C++ stripped  raw cut  stripped cut
coci21c1p3     23     49   177           115      87%           80%
coci21c1p4     24     38   239            76      90%           68%
coci21c1p5     34     41   189           102      82%           67%
coci21c2p1      8     11    59            25      86%           68%
coci21c2p2      9     13    82            48      89%           81%
coci21c3p1      9     16    63            29      86%           69%
coci21c3p2     20     27    80            46      75%           57%
coci21c6p1      7     16    62            32      89%           78%
coci21c6p2     19     22    71            41      73%           54%
coi07p1        12     17    59            28      80%           57%
total         165    250  1081           542      85%           70%
```

The cuts are for the v2 column. The honest number for what runs today is
the `today` column: 250 lines, 77% fewer raw and 54% fewer stripped. Part
of the cut is a different algorithm rather than the language: Kutije's
original builds a reachability table where the port unites boxes, and
Volontiranje's original keeps a segment tree per level where the port
keeps one stack.

## What read well

Kutije is the statement. Nine lines against 82, and at the full limits the
built form takes 0.01 s where the original takes 0.60 s:

```cp
var group = Dsu<Box>()
for _ in 0..m:
    let p: [Box]Box = read()
    for (i, to) in p.items: group.unite(i, to)
for _ in 0..q:
    let (a, b): (Box, Box) = read()
    print(if group.same(a, b): "DA" else: "NE")
```

Med ranks by comparing tuples, so "higher total first, then name" is one
expression and the original's sort-then-search function is gone:

```cp
let me = (-(total[i] + mine), rows[i].name)
1 + count(j for j in 0..n if j != i and (-(total[j] + theirs), rows[j].name) < me)
```

`?Int` with `min` that skips `none` carried the whole of Logičari's DP
without a `1e9` and without the original's "clamp at 1e9 so it does not
overflow" lines.

## K1. A `@memo` under an outer guess is hashed and never emptied

Logičari drops one cycle edge (a, b), guesses the colours of a and b, and
runs a tree DP for each of the four guesses. The natural port passes the
guess down:

```cp
@memo
fn blues(u: V, on: Bool, par_on: Bool, a_on: Bool, b_on: Bool) -> ?Int:
    ...
for a_on in Bool:
    for b_on in Bool: best min= blues(a, a_on, false, a_on, b_on)
```

That is right, and slow. At n = 10^5 the built form takes 0.75 s and
185 MB; the original takes 0.03 to 0.12 s and 30 MB with
`memo[maxn][2][2]` refilled by `memset` before each guess. The memo key
has five parts, two of which never change within one outer iteration, and
the prototype hashes it.

What one wants to write is the DP inside the loop, closing over the guess,
with a table that lives for one iteration:

```cp
for a_on in Bool:
    for b_on in Bool:
        @memo
        fn blues(u: V, on: Bool, par_on: Bool) -> ?Int: ...   // sees a_on, b_on
        best min= blues(a, a_on, false)
```

The cheat sheet does not say whether `@memo` may sit on a nested `fn` that
captures loop variables, nor when its table is dropped. This is a
collision of three features: `@memo`, closures, and "dense when every
parameter is a domain or `Bool`". Either the nested form is legal and
dense (table of 4n `?Int`, freed at the end of the iteration), or a memo
whose parameters are all `Index` shapes should be a flat array even with
five of them (here 16n slots, still small). Batch 28's D1 is the same
complaint from the other side.

## K2. Adding two `?Int` needs a helper

The DP adds child costs where either may be "impossible". `min` skips
`none`, which is what a minimum wants, but there is no spelling for "the
sum, or `none` if either is":

```cp
fn plus(x: ?Int, y: ?Int) -> ?Int: x? + y?

one = min(plus(one, off), plus(zero, lit))
zero = plus(zero, off)
```

`x? + y?` only works as the body of a function returning `?T`, because `?`
returns from the enclosing function; inline in the loop it would leave
`blues`. So every such DP grows a one-line `plus`. The prototype is
further away: `proto/cp.py build` stops at the v2 line with "expected an
expression (at ':')", and the runnable form is

```cp
fn plus(x: ?Int, y: ?Int) -> ?Int:
    if x == none or y == none: return none
    x! + y!
```

The asymmetry is the friction: under time pressure `min(a, b)` skipping
`none` suggests `a + b` does something sensible too, and it does not
compile. One fix that adds no construct is a prelude function, say
`both(x, y, f) -> ?T`, or documenting the `plus` one-liner in the cheat
sheet's optional section next to `min`.

Also in this port: `best min= blues(...)` where both sides are `?Int` is
rejected by the prototype ("expected Int, got ?Int"), though the cheat
sheet allows it; today it is `best = min(best, blues(...))`.

## K3. "The edge that closes the cycle" leans on evaluation order

A connected graph with n edges is a tree plus one edge. The v2 port finds
that edge with a `Dsu` and builds the tree from the rest:

```cp
var joined = Dsu<V>()
let (a, b) = first(e for e in edges if not joined.unite(e.0, e.1))!
let tree = Rooted<V>(Graph.undirected(e for e in edges if e != (a, b)), a)
```

Two things are uneasy. The filter of the comprehension has a side effect,
so it is only right if `first` pulls lazily and in order; the cheat sheet
does not promise either. And `e != (a, b)` removes the edge by value,
which is wrong if the same edge is listed twice (the statement forbids it,
but the port does not say so). Today's form is a loop that pushes into
adjacency lists or records the odd edge, eleven lines. A library function
that splits an edge list into a spanning forest and the rest would make
this one line and remove both doubts:

```cp
let (tree_edges, extra) = spanning(V, edges)    // wanted
```

The original spends 26 lines on a DFS cycle finder for the same job.

## K4. No operators on a small record

Set needs numbers a + b * omega. The original has a 60-line `cint` with
operators; v2 has records and free functions:

```cp
type Eis = (re: Int, w: Int)
fn add(x: Eis, y: Eis) -> Eis: (re: x.re + y.re, w: x.w + y.w)
fn turn(x: Eis) -> Eis: (re: -x.w, w: x.re - x.w)   // times omega

f[i + stride] = add(x, add(turn(y), turn(turn(z))))
```

against the original's `xa[i] + xb[i] * omega + xc[i] * omegasq`. The
cheat sheet lists `struct` and traits but shows no way to give `+` to a
user type, so the butterfly reads as nested calls. If the answer is "a
`struct` with `impl Add`", the cheat sheet should show it once; if the
answer is "no user operators", complex numbers, fractions and matrices
all read like this.

The prototype cannot pass the record through `add` cheaply either, so
today's form is two `Int` arrays and the six sums written out, which is
where a sign error would hide:

```cp
re[j] = xr - yw + zw - zr
w[j] = xw + yr - yw - zr
```

## K5. `I128` out of caution

The answer is a sum of 3^k cubes of numbers up to n. Each cube fits in an
`Int`, and so does the sum (it equals 3^k times a count of triples, below
2^58), but seeing that takes Parseval's identity. A range checker that
looks at `sum(x^3 for 3^k values up to 531441)` has to refuse, so the v2
port says

```cp
let triples = sum(I128(mul(x, mul(x, x)).re) for x in f) / size
```

and pays for 128-bit additions it does not need. The prototype has no
`I128`, and because the runnable form reads `n` as a plain `Int` it does
not bound the sum at all and accepts the 64-bit loop. So today the check
is either absent (unranged input) or too strict (ranged input); there is
no way to write "trust me, this fits" short of `unchecked`. No change
proposed; it is a case to keep for the overflow prover's test set.

## K6. A two-pointer's right end is one past the domain

Zemljište slides a window over column sums. The right end `r` is
exclusive, so it reaches m, which is not a column:

```cp
var (r, s) = (0, 0)             // s = sum of col[l..r]
for l in 0..m:
    while r < m and s < lo:
        s += col[r]
        r += 1
    offer(s)
    if r - l >= 2: offer(s - col[r - 1])
    s -= col[l]
```

With `domain Col = 0..m` and `col: [Col]Int`, `r` cannot be a `Col`, and
`r - l` and `col[r - 1]` need conversions. The port therefore indexes by
plain ranges (`[0..m]Int`) and gives up the named domain; on the
prototype the attempt to keep the domain and index it with a counter
stopped at "[Col]Int has no `.raw`". `slide_until` in the cheat sheet
yields `(lo, hi)` pairs, but this loop also needs the window's sum and the
element just before `hi`, so it did not fit. Every half-open two-pointer
has this shape; a worked example in the cheat sheet using boundaries
(`v.r`, `b.left`) would show the intended spelling.

## K7. "Position to insert at" is `first_ge(x) ?? len`

The patience step of Volontiranje wants the index where a value goes,
which may be the end:

```cp
let at = tails.first_ge(p[i]) ?? tails.len
if at == tails.len:
    tails.push(p[i])
    level.push([])
else: tails[at] = p[i]
level[at].push(i)
```

`first_ge` returning `?I` is right for lookups, but for insertion the
`none` is immediately turned back into `len` and then tested again. The
branch could go the other way (`if let at = tails.first_ge(...)`), except
that both arms then need `level[...].push(i)` with a different index.
`xs.count_lt(x) -> Int` (the rank) would say it directly. The prototype
has no `first_ge` on a `List` ("List<Int> has no method `first_ge`"), so
today it is a seven-line binary search, and `level.push([])` needs a
typed temporary:

```cp
let empty: List<Pos> = []
level.push(empty)
```

## K8. Text output: no `{t:.3f}`, no `join`, no `Str + Str`

Two ports are about printing. Kaučuk prints `1.2.3 title`:

```cp
print(".".join("{at[d]}" for d in 1..=depth), title)
```

and COI 2007 prints three decimals with `print("{t:.3f}")`. Neither runs:

- `proto/cp.py build` has no string interpolation (it prints the braces
  literally), no `F64` ("unknown type Float"), and rejects string
  concatenation ("`+=` between Str and Str").
- `proto/cp.py lib-run` interpolates but ignores the format: `"{t:.3f}"`
  printed `13.099999999999998` where the original prints `13.100`. Its
  float type is spelled `Float`, not `F64` ("unknown type `F64`").

Today's Kaučuk is three `print`s, one per depth, and today's COI 2007
rounds by hand:

```cp
let th = Int(t * 1000.0 + 0.5)
let frac = th % 1000
let pad = if frac < 10: "00" elif frac < 100: "0" else: ""
print("{th / 1000}.{pad}{frac}")
```

This is a prototype gap, not a design one, but it means no floating-point
port can be checked exactly and none can be built to C++. `Int(x)` on a
float is the interpreter's floor; v2 makes that an error in favour of
`floor(x)`, which the interpreter does not have ("unknown name `floor`").

## K9. Smaller prototype gaps met on the way

Each of these is v2 as the cheat sheet has it, with the prototype's reply:

- Tuple and record reads, `let (n, k): (Int, Int) = read()` and
  `[0..n](name: Str, rounds: [0..5]0..=500)`: `proto/cp.py build` says
  "unknown name read", which points at the wrong thing. One scalar per
  line today, which is most of the gap between the `v2` and `today`
  columns for the easy problems (Med: 7 lines against 16).
- A filtered loop over a neighbour list, `for v in g[u] if v != parent`
  with `parent: ?V`: "expected ':' (at 'if')". Today
  `if parent == none or v != parent!`, which is exactly the line a
  `?V` parent was supposed to remove.
- `Graph.undirected`, `Rooted` and a bare `Dsu<V>()` do not exist in the
  compiler; `Dsu` needs `use std.dsu.{Dsu}` and `Dsu<Box>.new()`.
- A `Str` cannot be indexed or iterated ("cannot index Str"); today the
  card is read as `[k]Char`.
- `for x in Bool` is read off the cheat sheet's statement that `Bool` is
  an `Index` shape; no example shows it. Today `for x in [false, true]`.
- `(0..size).step(3 * stride)` is a `while` today.

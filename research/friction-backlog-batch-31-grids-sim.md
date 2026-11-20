# Friction backlog, batch 31: grids, BFS, flood fill and simulation

Batch 31 ports twelve grid, state-search and simulation problems against the
locked v2 spec (`docs/one-page-cheat-sheet.md`). The ports are in
`proto/ports/scout_batch_31/`: the top-level files are the v2 forms, and
`today/` holds the forms that run on the prototype and match the original
C++ on 150 random cases each (`check.py`).

This note answers one question per locked feature: did it make the messy
problem easy to write, and what was still awkward? Items are numbered `31.n`
so they can be cited without colliding with earlier batches' letters. Markers
in the ports (`// FRICTION D1`, `F1`, `S2`, `X1`, `N1`-`N3`) point at the items
below.

## Verdict on the locked features

### `p.adj4` and `p.adj8`: the biggest win of the batch

Six of the twelve problems walk the four neighbours of a cell. Every original
spells that as two offset arrays, a loop to four, two additions and a bounds
check. With `p.adj4` the grid Dijkstra of `ucrpc21f` is its statement:

```cp
let dist = dijkstra(p => (q, if g[q] == 'b': 1 + t else: 1) for q in p.adj4,
                    from: [(Row.first, Col.first)])
print(dist[Row.last, Col.last]!)
```

and the IMO checker's path count has no bounds logic at all:

```cp
for p in sorted(Cell, by: p => a[p]):
    let lower = [q for q in p.adj4 if a[q] < a[p]]
    ways[p] = if lower.is_empty: 1 else: sum(ways[q] for q in lower)
```

Nothing to change here. No problem in the batch wanted `adj8`; the one
eight-direction problem (`noi10p3`) needs the directions in rotational order
with a running cost, so it uses `Dir8` and `p.step(d)` instead (item 31.6).

The prototype does not have `adj4` yet (`(f0: Row, f1: Col) has no field
adj4`), so five `today/` files carry the same six-line helper:

```cp
fn adj4(p: Cell) -> List<Cell>:
    var out: List<Cell> = []
    if let r = p.0 - 1: out.push((r, p.1))
    if let r = p.0 + 1: out.push((r, p.1))
    if let c = p.1 - 1: out.push((p.0, c))
    if let c = p.1 + 1: out.push((p.0, c))
    out
```

That helper is the single largest source of the gap between the v2 and
`today/` line counts, so `adj4` is the first thing worth graduating.

### `[R, C]T`: right every time, including past two dimensions

`let g: [Row, Col]Char = read()` read every picture in the batch with no
loop, and `g[p]` with `p: Cell` removed all the `[x][y]` pairs. The product
form scaled without surprises: `dmopc19c4p2` keeps its distances in
`[Row, Col, Row, Col]?Int` and `noi10p3` in `[Ext, Ext, Dir]?Int`, and both
run today.

One thing was felt in every state-space search: a table indexed by a state
that is itself a tuple of cells has to be indexed by its flattened parts
today.

```cp
let cur = q[head]                       // (token: Cell, hole: Cell)
let d = dist[p.0, p.1, h.0, h.1]!       // today
let d = dist[cur]!                      // v2: [State]T, State = (Cell, Cell)
```

The cheat sheet already promises `[Cell]T` and tuple indices, so this is a
prototype gap rather than a design hole; the search states in this batch
(`(Cell, Cell)`, `(V, Met)`, `(Cell, Dir8)`) are a good test set for it.

### `Str` indexed `0..len`: fine in the spec, unusable on `proto/cpc`

Only `coci25c3p1` has a real string: a command tape of unknown length. The
v2 form is short and obvious:

```cp
let moves: Str = read()
for ch in moves:
    if let d = Dir4.from("UDLR", ch):
        at = at.step(d) ?? at
    else:
        for p in Cell:
            if abs(p.0 - at.0) + abs(p.1 - at.1) < k: g[p] = ch
```

On the prototype a `Str` can be read and printed and nothing else: `for ch in
s` gives `cannot iterate Str`, `s[i]` gives `cannot index Str`, and
`List<Char> = read()` expects a length first. The interpreter does iterate a
`Str`, so this one port is checked with `cp.py lib-run`. Grid problems mostly
dodge the issue because `[Row, Col]Char` reads the picture directly, which is
worth knowing: the `0..len` decision is barely exercised by grid work, and
needs the string batches to judge it.

### `a.fill(x)`: reads right, but invites a slow reset (F1)

One port wanted a whole-table reset. `ucrpc21g` tracks the reachable
`(cell, coins held)` states tick by tick, and needs a "seen this tick" table:

```cp
var seen: [Cell, 0..=coins]Bool = false
for t in Tick:
    seen.fill(false)
    ...
        if not seen[q, have]:
            seen[q, have] = true
            next.push((q, have))
```

That is the obvious code and `fill` makes it one line. It also clears
`n * n * (coins + 1)` cells on each of 105 ticks, up to 10^9 writes at the
limits, when only the states touched last tick need clearing. The form that
runs today avoids it with a stamp table, which is the standard contest trick
and has no spelling in the spec:

```cp
var stamp: [Row, Col, 0..=coins]Int = -1
...
        if stamp[q.0, q.1, have] != t.idx:
            stamp[q.0, q.1, have] = t.idx
```

So `fill` is fine as a primitive, but the pattern "a visited table reused
across rounds" deserves a stdlib type, for example `Marks<D>()` with
`.mark(i) -> Bool` (true when newly marked) and an O(1) `.clear()`. The
other simulation that looked like a `fill` user (`sac22cc1p3`) wants a
snapshot instead: water falls on the board as it stood before the pass, so
the natural code collects the moves first and applies them after. `fill` is
not in the prototype yet (`[Row, Col]?Int has no method fill`).

### `(a, b) = (x, y)`: reads well for cell swaps

Simulations move things between two cells, and the parallel form says so in
one line with no temporary:

```cp
(g[l], g[at]) = ('W', '.')                       // water runs left
(g[p], g[p.step(Dir4.S)!]) = ('.', 'W')          // water falls
(below, below_row) = (node((r, c)), r.to_idx)    // column sweep in cco19p2
```

The prototype rejects it (`a tuple is a value, not a place to assign to`), and
`swap(mut g[a], mut g[b])` is rejected too because the two places may alias,
so `today/` writes two assignments. One question the spec should answer in
its own words: are `(g[p], g[q]) = ...` targets allowed when `p` may equal
`q`? Right-hand side first, then left to right, is the unsurprising rule.

### Bare-core fallback: always available, and never ugly

Every port has a `today/` form that uses only domains, `?T`, `List`,
`MinHeap` and `if let`, and none needed a workaround that changed the
algorithm. The fallback costs lines, not clarity: 334 lines against 161 for
the v2 forms of the ten new problems, with the `adj4` helper, a hand-rolled queue (`List` plus a
`head` index, since `Deque` is missing) and hand-written search loops in
place of `bfs`/`dijkstra` accounting for most of the difference.

## Remaining friction

### 31.1 A sub-lattice of a canvas has no name (D1)

`ucrpc21d` draws `n x m` rooms on a `(2n+1) x (2m+1)` canvas: rooms on odd
cells, doors between them. The rooms are the real vertices, but they are
addressed through canvas coordinates:

```cp
for i in (1..2 * n).step(2):
    for j in (1..2 * m).step(2):
        let p: Cell = (Row(i), Col(j))
        for d in Dir4:
            if let door = p.step(d) and let q = door.step(d):
                if not wall(door): rooms.unite(p, q)
```

`Row(i)` is a checked conversion that can never fail here, and `Dsu<Cell>`
allocates four times the nodes it needs. Double-resolution ASCII art is a
recurring contest input shape. A small stdlib view would cover it, for
example `g.lattice(2, 1)` giving a domain of room cells with `room.canvas`
and `room.wall(d)`. Worth a bake-off before adding anything.

### 31.2 Running two effectful steps "until neither changes" (S2)

```cp
while run_left() | fall(): pass
```

Both steps must run every round, so `or` is wrong, and it is wrong silently:
with `or` the water stops falling whenever it moved left. The prototype form
binds both results first:

```cp
var moving = true
while moving:
    let a = run_left()
    let b = fall()
    moving = a or b
```

The prototype rejects `|` on `Bool` (`| needs Int operands`). Either define
a non-short-circuit `|` on `Bool` and say so in the cheat sheet, or give this a name such as `until_stable(run_left, fall)`. The
original C++ uses `changed |= ...`, so people will reach for `|`.

### 31.3 No way to stop a program early (X1)

`dmopc21c6p4` prints `-1` and stops from two places. There is no top-level
`return` (`return outside a function`) and no `exit`, so the whole program
moves into a function:

```cp
fn solve() -> ?List<Int>:
    if any(c in [e.u, e.v] and d in [e.u, e.v] for e in edges): return none
    ...
if let colours = solve():
    print(2)
    for x in colours: print(x)
else:
    print(-1)
```

Returning `?T` and printing once is arguably the better shape, but "print the
verdict and stop" is among the most common lines in contest code, and the
cheat sheet does not say how to write it. Pick one answer and list it: either
top-level `return`, or a `done(x)` that prints and ends the test case (which
also has an obvious meaning under `@multitest`).

### 31.4 Edge data stored per direction is four ragged arrays (N1)

`noi10p3` gives a flow for every directed street segment as four blocks of
different shapes: `(n+1) x n`, `n x (n+1)`, `(n+1) x n`, `n x (n+1)`.
Reading them is pleasant, since anonymous ranges give each block its own
shape:

```cp
let east: [X, 0..n - 1]Int = read()
let south: [0..n - 1, Y]Int = read()
let west: [X, 1..n]Int = read()
let north: [1..n, Y]Int = read()
```

Using them is not: every access needs a direction switch and a hand-written
edge test, with mixed `X`/`Int` indices.

```cp
fn flow(p: Cell, d: Dir4) -> Int:
    let (x, y) = (p.0.to_idx, p.1.to_idx)
    match d:
        Dir4.E: if y < n - 1: east[p.0, y] else: 0
        ...
```

What the code wants is `[Cell, Dir4]Int` with a default for edges that leave
the grid. Reading into it needs something like
`read_edges(Dir4.E)`, or simply a documented idiom. Grid problems with
weights on edges (rather than cells) all hit this.

### 31.5 `Row + Int` is optional even when the code just proved it fits

`cco19p2` moves every block down by its shape's drop, which the shortest-path
argument guarantees stays on the board:

```cp
out[(r + drop[node((r, c))]!)!, c] = '#'
```

Two `!` on one index, for two different reasons, is hard to read. The same
happens for a row walked backwards on the prototype (`Row(n - 1 - i)`, where
v2 has `Row.rev`). A named local helps (`let landed = (r + fall)!`), and that
is probably the advice to document, but the range checker could also accept
`r + k` as `Row` under an `assume`/`invariant` that bounds `k`.

### 31.6 Direction sweeps with a running cost (N2, N3)

`noi10p3` turns from the entry direction through six of the eight compass
directions, paying a flow each time it crosses an axis direction. `Dir8` with
`.ccw` and `.back` expresses the turning well. Two pieces were missing and
are invented in the v2 port: a way to ask whether a `Dir8` is an axis
direction (`dir.as_dir4 -> ?Dir4`), and a way to iterate "the next k
directions after d" without a counter and a manual `dir = dir.ccw`. Something
like `d.ccw_from.take(6)` would remove the counter. Low priority: one
problem, and the fallback (integers mod 8 with two offset tables) is short.

### 31.7 The domain rules differ between the compiler and the interpreter

`abs(i - r)` on two `Row` values compiles under `proto/cpc` and is an error
under the interpreter (`a Row is a position, not a number`), which wants
`i.to_idx - r.to_idx`. The cheat sheet says `b - a -> Int`, so the compiler is
right. The same port also needs `g.row(r)` on one side and a `Str` built with
`push` on the other, because the interpreter has no `row` and the compiler
has no `Str.push`. A port cannot currently be written once and run on both.

### 31.8 Printing a picture is fine; building one from a state is not

`for r in Row: print(g.row(r))` covered every output in the batch. The gap is
the step before it. `cco19p2` and `coci20c6p1` both build an output picture
from a rule per cell, and write a mutable grid plus loops:

```cp
var out: [Row, Col]Char = '.'
for (r, c) in Cell:
    if g[r, c] == '#': out[(r + drop[node((r, c))]!)!, c] = '#'
```

This scatter form (write to a computed cell) has no comprehension
equivalent, and that seems right; no change proposed. Noted because it is the
one place where the ports still look like the C++.

## Prototype gaps met in this batch

For whoever schedules prototype work, in the order they cost lines: `p.adj4`
and `p.step(d)`; `Deque`; tuple-typed array indices
(`dist[state]`); parallel assignment; iterating and indexing `Str`; `read()`
as an expression inside a loop; `a.fill(x)`; top-level early exit. None of
them blocked a port.

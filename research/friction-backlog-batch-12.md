# Friction backlog, batch 12: constructive, ad-hoc and grid simulation

A Stage-1 scout pass over ten constructive, ad-hoc, grid-simulation and
implementation-heavy submissions. The ports are in
[`proto/ports/scout_batch_12/`](../proto/ports/scout_batch_12/): ccc18s3,
coci10c7p4, wossoly21s3, dmopc21c6p3, noip09p4, noi05p1, ccc10s4,
ioi05p1, cco02p3 and cco19p1. This file follows the format of
[`friction-backlog.md`](friction-backlog.md) (batch 1). Its items are
numbered `L0`... so they do not collide with `F` (batch 1), `B` (batches
2 and 4), `G` (batch 3), `K` (batch 5), `S` (batch 6), `D` (batch 7) or
`H` (batch 8). An item an earlier batch already has is named with its
number and only what is new is added.

What was checked: nine of the ten ports build with `proto/cpc` at commit
`91fb701` and agree with their originals on 200 small random cases each
(`proto/ports/scout_batch_12/check.py`); cco02p3 is interactive and is
checked through a judge. cco19p1 is a sketch that was never run. No port
was run at full size. Every "today" snippet below is a line a port
contains. Every error message quoted was produced by a small probe with
`proto/cp.py build` at `91fb701`. The "wanted" snippets do not build. A
count such as "4 of 10" is the number of the ten ports with the awkward
line. Corpus counts are from a regular expression over the 3462 `.cpp`
files in `corpus/submissions/` (files, not problems), so they are
estimates, and the pattern is named each time.

The batch was started at `5aca634`. `54269ec` landed while it ran and
removed three items before they were written down: `e?` inside an
expression, `min` / `min=` over a mix of `Int` and `?Int`, and
`use std.dsu.{Dsu}`. The ports use all three. Numbers L10 and L21 are
unused for that reason.

The batch was rebased onto `9f09d6b` before it was pushed. `check.py`
passes there, and the probes for L0, L26, L5, L1 and L4 give the same
results; the other probes were not run again.

## The one-paragraph version

Seven of the ten are grid problems, and the grid is where the prototype
is furthest behind the plan. `plan/12` already says `domain Cell = (Row,
Col)`, `enum Dir4` and `p.step(d)`. None of the three builds, so every
grid port opens with the same five lines of `DIRS` and `step`, loops
over cells through a helper, and writes its search by hand because
`std.graph.bfs` only takes a domain and a cell is not one. L5 and L1
together remove about a third of the code of six ports. L0 is a
memory-safety bug that the hand-written worklists walk straight into.

## Suggested order for a Stage-2 sprint

1. L0 and L26: a `let` that is a reference, and letters read as the
   first variant of an enum. Both give wrong answers without a message.
2. L5: a pair of positions is a position. 7 of 10 are waiting on it, and
   it is what lets L7 be library code.
3. L1 and L2: moving on a grid, and directions that come from the input.
   6 of 10.
4. L7 and L4: searches from many sources, with weights 0 and 1, and the
   queue they need. 4 of 10.
5. L14: a step that is undone on the way back. 2 of 10, and every
   backtracking search.
6. L3, L6, L8, L20, L22: one to three ports each, all small.
7. L27: an interactor next to the program.
8. The rest (L9, L11 to L13, L15 to L19, L23 to L25): one port each.

## Bugs

### L0. A `let` that takes a pair out of a list is a reference (fixed in Stage-2 sprint #2; found porting wossoly21s3)

*Checked with `proto/cpc`, `g++ -fsanitize=address`:*

```cp
var g: [0..3](Int, Int) = (1, 2)
let (a, b) = g[0]
g[0] = (5, 6)
print(a, b)
```

prints `5 6`. A `let` changed its value. With a list it is worse:

```cp
var xs: List<(Int, Int)> = [(1, 2)]
var next = 0
while next < xs.len:
    let (a, b) = xs[next]
    next += 1
    if xs.len < 100: xs.push((a + 1, b + 1))
    if a != b - 1: print(next, a, b)
print(xs.len)
```

builds without a message, prints lines of garbage, and under
AddressSanitizer stops with `heap-use-after-free ... in main`. The
emitted C++ is `const auto& _d1 = xs[...]; const auto& a = _d1.f0;`, so
`a` and `b` point into the vector and the `push` moves it. The program is
[`proto/ports/scout_batch_12/bugs/let_then_push.cp`](../proto/ports/scout_batch_12/bugs/let_then_push.cp).

This is the loop of every breadth-first search written with a list and a
head index, which is what a port writes today because there is no queue
(L4). *Checked:* `let p = xs[next]` without the pattern is a copy and is
fine, `if let (a, b) = xs[next]` is fine, and `for x in xs` with a push
in the body is already a compile error with a good hint. Only the
destructuring `let` is wrong. Proposal: a destructuring `let` copies,
the same as a plain one. The differential fuzzer should have a case that
pushes to a list between a `let` and its use.

*Fixed.* A destructuring `let` copies, as a plain one does, and so does the subject of a `match`, which had the same reference (decision 15.4). Both programs above are in `crates/cpl-cli/tests/cases/redteam/let_destructure_push.cp`, built under ASan; `bugs/let_then_push.cp` prints `100` and nothing else.

### L26. Letters read into an enum without `@tags` are all its first variant (fixed in Stage-2 sprint #2; probe for cco19p1)

*Checked with `proto/cpc`:*

```cp
enum Tile:
    X
    O
let n: 1..=3 = read()
domain Row = 1..=n
let g: [Row, Row]Tile = read()
print(count(g[r, c] == X for r in Row for c in Row))
```

On `2 XO OX` this prints 4. The enum is read as a number, the read
fails, and every cell is variant 0. With `@tags("X", "O")` above the
enum the same input prints 2 and a wrong letter stops with `unknown
tag`. Proposal: reading an enum with no fields and no `@tags` reads its
variant names, and anything else is an error at the read, never a
default. Two smaller things from the same probe: the one-line form `enum
Way: N | E | S | W` of `plan/12` gives `expected NEWLINE (at 'N')`, and
`[Way]Int` gives `array dimension must be a domain` (batch 4 B11).

*Fixed.* Reading an enum with no fields and no `@tags` is a compile error that gives the `@tags(...)` line to add (`crates/cpl-cli/tests/cases/bad/enum_read_no_tags.cp`), and a local run stops on a word where a number is expected instead of reading 0 (`crates/cpl-cli/tests/cases/redteam/read_word_as_int.cp`); decision 15.9. Reading variant names without `@tags`, the one-line `enum` and `[Way]Int` are left to the language pass.

## Grids

### L5. A pair of positions is not a position (7 of 10)

`plan/12` says `domain Cell = (Row, Col)` and `plan/19` says a tuple of
`Index` is an `Index`. In the prototype only `g[p]` knows that.

*Checked with `proto/cpc`,* with `type Cell = (Row, Col)`:

- `domain Cell = (Row, Col)` gives `domain needs a range`.
- `for p in Cell` gives `cannot iterate type (f0: Row, f1: Col)`.
- `var seen: [Cell]Bool` gives `array dimension must be a domain, a range,
  or a struct with impl Index`.
- `Dsu<Cell>()` gives `Dsu<V> needs a domain`.
- `bfs(p => around(p), from: [start])` gives `bfs(adj, from) starts from
  a list of vertices, got List<(f0: Row, f1: Col)>`.

So today ccc18s3 and coci10c7p4 carry a helper,

```cp
fn cells() -> List<Cell>: [(r, c) for r in Row for c in Col]
for cam in cells():
```

and ccc18s3, coci10c7p4, wossoly21s3 and dmopc21c6p3 each write their own
search loop with a `dist` or `seen` grid and a list, although
`std.graph.bfs` is exactly that loop. Proposal: nothing new, only the
rule the plan has. A tuple of domains is a domain, row-major, wherever a
domain is asked for: a `for`, an array dimension, a type argument of
`Dsu`, `bfs`, `dijkstra`, `SegTree`. (`Set<Cell>` and `Map<Cell, T>`
already build.) Then the search of
wossoly21s3 is one line of library code,

```cp
fn walk(from: Cell, to: Cell) -> ?Int: bfs(p => (land(q)? for q in p.around()), from: [from])[to]
```

and its 14-line `walk` goes away. 7 of 10 use a cell as a value; 4 of 10
would drop a hand-written search. Corpus: `queue *< *pair` is in 46
files, nearly all of them a grid search.

### L1. Moving on a grid (6 of 10)

Today, at the top of ccc18s3, wossoly21s3, dmopc21c6p3 and cco02p3:

```cp
const DIRS = [(-1, 0), (1, 0), (0, -1), (0, 1)]
fn step(p: Cell, d: (Int, Int)) -> ?Cell:
    if let r = p.0 + d.0 and let c = p.1 + d.1:
        return (r, c)
    return none
```

and in coci10c7p4, which moves to eight neighbours, a nine-cell loop
that relies on a cell being one of its own neighbours doing no harm.
*Checked:* `Dir4` is an `unknown name`, `p.step(d)` and `p.around()` are
`no method`, and `p + (1, 0)` gives `+ needs Int operands`. The good
part already works: `p.0 + d.0` is a `?Row`, so the bounds check is in
the type and none of the six ports has an `if 0 <= r and r < n`.

Wanted, as userland in a `std.grid` and generic over any two domains:

```cp
for d in Dir4:
    if let q = p.step(d) and g[q] != 'W': ...
for q in p.around(): ...             // the cells one step away that exist
for q in p.around(Dir8): ...         // eight ways
```

`step` is in `plan/19` §3. What is new here is `around`, which is what
five of the six loops want (they never use `d` again), and that the set
of moves is a value: `Dir4`, `Dir8`, or a port's own list of offsets
(knight moves). For the library to be written in CP, one language rule
is missing: a pair of positions plus a pair of `Int`s is an optional
pair, `(r, c) + (dr, dc)` being `(r + dr, c + dc)` with `none` if either
is. That is `+` on tuples by fields, which is not special to grids. 6 of
10. Corpus: a `dx` / `dy` table (`d[xyrc]\[[0-9]*\] *= *\{` and two
similar spellings) is in 102 files.

### L2. A direction that comes from the input (3 of 10)

Today, ccc18s3 turns a conveyor letter into an offset with four `if`s,
noi05p1 keeps `way: 1..=4` and compares it with 1, 2 and 3, and cco02p3
does

```cp
var face = 0
if facing == 'E': face = 1
if facing == 'S': face = 2
if facing == 'W': face = 3
...
face = (face + 1) % 4
at = step(at, DIRS[face])!
```

Wanted: the mapping written once, where the directions are declared, and
turning as a method:

```cp
let belt = Dir4.from("ULDR", g[p])         // ?Dir4: none for any other letter
let tilts: [Tilt](from: Int, to: Int, way: Dir4.numbered(N, S, W, E)) = read()
var face: Dir4 = read()                    // N E S W by name
face = face.cw
```

Proposal: two general pieces. An enum is a cyclic domain, so `d.cyc + 1`
is the next variant round the circle (batch 1 F11) and `cw` / `ccw` /
`back` are one-line methods of `Dir4` in userland. And any enum without
fields can be built from a letter or a number through an explicit
table: `E.from(letters, ch) -> ?E` as a prelude fn, and `@tags` (which
exists, see L26) for the reading side. No direction-specific compiler
code. 3 of 10.

### L3. Lines of sight, and whole rows walked in a direction (2 of 10)

Today, the camera of ccc18s3:

```cp
var at: ?Cell = cam
while let q = at:
    if g[q] == 'W': break
    if belt(g[q]) == none: seen[q] = true
    at = step(q, d)
```

`while let q = at and g[q] != 'W'` gives `unknown name q` (batch 6 S6).
And noi05p1 needs every row or column of the hall in the direction of
the tilt, which today is

```cp
fn lines(way: Int) -> List<List<Cell>>:
    if way == 1: return [[(r, c) for r in Row.rev] for c in Col]
    if way == 2: return [[(r, c) for r in Row] for c in Col]
    if way == 3: return [[(r, c) for c in Col.rev] for r in Row]
    return [[(r, c) for c in Col] for r in Row]
```

Wanted:

```cp
for q in cam.ray(d).take_while(q => g[q] != 'W'): if g[q] not in "UDLR": seen[q] = true
for line in Cell.lines(way): ...
```

Proposal: in `std.grid`, `p.ray(d)` yields `p`, `p.step(d)`, ... until
the edge, and `(A, B).lines(d)` yields each maximal ray. Both are
generators over `step`, so they come free with L1; `take_while` on any
generator is the general piece (batch 3 G21 has `take`). 2 of 10.

### L6. Where is the `S` (3 of 10)

Today ccc18s3 scans every cell for its start, coci10c7p4 scans for `P`
and for every `K`, and wossoly21s3 builds a `Map<Char, Cell>` of the
whole grid to find ten letters. Wanted:

```cp
let start = g.find('S')!
all(seen[p] for p in town.where('K'))
```

Proposal: on any array, `a.find(x) -> ?I` (the first position holding
`x`) and `a.where(x)` / `a.where(pred)` (every such position, as a
generator). They are array methods, not grid methods: `[V]Int` wants
them as often. *Checked:* `g.find('S')` gives `no method find`, and
`first(...)` over a generator is an `unknown name`. 3 of 10.

### L22. A grid seen mirrored or turned (ioi05p1, 2 lines; batch 3 G2, batch 8 H15)

ioi05p1 runs one scan four ways. Today:

```cp
let down = above([R, C] (r, c) => garden[R.from_idx(R.len - 1 - r.idx), c])
let along = split([Col, Row] (c, r) => count[r, c])
```

Wanted: `above(garden.flipped)` and `split(count.transposed)`. *Checked:*
`g.transposed` gives `no .transposed`. Proposal: `flipped` (G2, on the
first dimension), `transposed`, and `r.mirror` for the position the same
distance from the other end, which is what the first line spells with
`from_idx` and `.idx`. As views they cost nothing and compose, so the
four orientations of a grid are `g`, `g.flipped`, `g.transposed`,
`g.transposed.flipped`. What already worked well: `fn above<R, C>(garden:
[R, C]Int) -> [R]?Int` is generic over the two domains, so the transposed
call type-checks with rows and columns swapped. 1 of 10.

### L12. The block a cell is in (noip09p4)

Today:

```cp
fn box_of(r: Line, c: Line) -> Box: Box(r.idx / 3 * 3 + c.idx / 3)
```

*Checked:* `r / 3` on a position gives `/ between Line and Int`.
Wanted: `(r, c).block(3, 3)`, a position in the domain of 3 by 3 blocks.
Proposal: `D.blocks(k)` is a domain (the stretches of `k` positions,
which `D.windows(k)` already walks), `i.block(k)` the block of `i`, and
on a pair it works by fields, so with L5 the box of a sudoku cell is a
`(Line.blocks(3), Line.blocks(3))` and needs no arithmetic. The same
thing is the bucket of a sqrt decomposition. 1 of 10.

## Searches

### L7. A search from many sources, and one with free moves (4 of 10)

With L5 a plain search is `std.graph.bfs`. Three of the four want a
little more than distances.

coci10c7p4 only wants what is reachable:

```cp
var seen: [Row, Col]Bool = false
var todo: List<Cell> = []
for p in cells(): ...
while let p = todo.pop():
    for q in around(p):
        if not seen[q] and lo <= high[q] and high[q] <= hi:
            seen[q] = true
            todo.push(q)
```

dmopc21c6p3 wants to know which source got there first, sources in a
given order, and to stop after `k` rounds; today that is a `wave` list
rebuilt once a round. ccc18s3 has moves that cost 0 (a conveyor) and
moves that cost 1, written today with two lists `near` and `far`.

Wanted:

```cp
let seen = reach(p => (q for q in p.around() if ok(q)), from: town.where('P'))     // coci10c7p4
let first = bfs(p => p.around(), from: painted, limit: k)                           // dmopc21c6p3
paint[q] = paint[first.source[q]]
let dist = shortest(p => moves(p), from: [(start, 0)], w: m => m.cost).dist         // ccc18s3
```

Proposal, all in `std.graph` and none of it about grids: `reach(adj,
from) -> [V]Bool`; `bfs` gains `limit:` and returns `source` beside
`dist` (the sources are tried in the order given, which is the tie
rule); and `shortest` picks a deque when every weight has type `0..=1`,
which the range types can tell it. 4 of 10.

### L4. No queue (3 of 10; batch 2 and batch 6 name `Deque`)

*Checked:* `Deque<Int>` is `unknown type`, `xs.pop_front()` is `no
method`. So wossoly21s3 has `todo` and `next`, noi05p1 has `reach` and
`head`, and ccc18s3 has `near` and `far`. The list-and-head form is the
one L0 breaks. Proposal: `Deque<T>` in userland over a ring buffer with
`push`, `push_front`, `pop`, `pop_front`, `first`, `last`. Once L7 is in,
a port should rarely need it by name. 3 of 10. Corpus: `deque *<` is in
65 files.

### L14. A step that is undone on the way back (2 of 10)

Today, noip09p4:

```cp
mark(r, c, d, true)
board[r, c] = d.label
fill(score + d.label * worth(r, c))
board[r, c] = 0
mark(r, c, d, false)
```

and cco02p3, where the undo is four commands to a robot:

```cp
if forward():
    explore()
    right()
    right()
    forward()
    left()
    continue
```

Both are "do, recurse, undo", and the undo is the part that is
forgotten or put after an early `return`. Wanted:

```cp
with placed(r, c, d): fill(score + d.label * worth(r, c))
with stepped(): explore()
```

Proposal: no new mechanism. `plan/19` §3.4 already has a fn whose body
runs code, hands control to the caller once, and runs more code when the
caller is done: a projection, with "code after the `lend` runs when the
caller is done". A projection that lends nothing is a scope guard:

```cp
fn placed(r: Line, c: Line, d: Digit) lends ():
    mark(r, c, d, true)
    lend ()
    mark(r, c, d, false)
```

What is needed is the statement form `with f(...): body` for a
projection whose lent value is not used, and the guarantee that the tail
runs on `return`, `break` and `?` out of the body. cco02p3 shows the
condition matters too (`forward()` may fail), so a projection must be
allowed to not lend: `with stepped(): ...` skips its body when the step
did not happen. 2 of 10 here, and every backtracking search.

### L13. The best candidate, or none left (noip09p4; batch 5 K7 has `argmin`)

Today, nine lines: a `pick: ?(Line, Line)`, a `fewest = 10`, two loops
and a comparison. Wanted:

```cp
let (r, c) = min_by(board.where(none), p => options(p)) else:
    best max= score
    return
```

Proposal: `min_by(gen, key) -> ?T` and `max_by`, `none` on an empty
generator, in the prelude beside `min` and `max`. *Checked:* `min_by` is
an `unknown name`. What already works well here: `let ... else`, and
`best max= score` on a `?Int`. 1 of 10.

## Sequences

### L8. For each left end, the first right end that is enough (2 of 10; batch 6 S12)

Today, coci10c7p4:

```cp
var top = 0
for floor in 0..levels.len:
    while top < levels.len and not delivers(levels[floor], levels[top]): top += 1
    if top == levels.len: break
    best = min(best, levels[top] - levels[floor])
```

and ioi05p1 the same with a running sum, `C.from_idx(right)` and
`C.from_idx(left)` because `left` and `right` have to be `Int`s to be
compared with `C.len`. Wanted:

```cp
for (lo, hi) in levels.stretches(delivers): best min= hi - lo
for w in C.stretches(w => sum(strip[w]) >= k): if sum(strip[w]) == k: least min= ...
```

Proposal: `xs.stretches(enough)` on any list or domain: for each left
end in order, the shortest stretch from it for which `enough` holds,
given that `enough` stays true when a stretch grows; both ends only move
right. A second form takes a monoid and keeps the running fold, so
`sum(strip[w])` is not recomputed (`stretches(sum, s => s >= k)`). Batch
6 S12 is the same loop draining a sorted list from both ends; one
abstraction should cover both. 2 of 10.

### L15. The largest of the last few (noi05p1; batch 5 K14)

Today, eight lines of a monotone queue kept in a list with a head index
(the port quotes them; they are the body of noi05p1). Wanted:

```cp
for run in line.split(p => hall[p] == 'x'):
    let best = sliding(max, [far[p]? - i for (i, p) in run.enumerate()], seconds + 1)
```

Proposal: `sliding(op, xs, width)` for any idempotent `op` from
`std.algebra` (`min`, `max`, `gcd`), giving for each place the fold of
the last `width` values, `none` skipped; and a `MonoQueue<T>` under it
for the cases that push and pop by hand. Batch 5 K14 asks for the same
over a grid window: K14 is `sliding` applied to rows and then columns.
1 of 10.

### L16. Stretches between obstacles (noi05p1)

Today the queue is reset by hand when the walk meets furniture:

```cp
if hall[p] == 'x':
    reach = []
    head = 0
else: ...
```

Wanted: `for run in line.split(p => hall[p] == 'x')`. Proposal:
`xs.split(pred)` on any list or generator, yielding the maximal runs
between elements that satisfy `pred`. *Checked:* `List<Int> has no
method split`. Batch 3 G12 and batch 6 S15 group by a key; `split` is
the other half (cut at, rather than group by). 1 of 10.

### L18. Consecutive pairs round a ring (ccc10s4; batch 1 F11)

Today:

```cp
for j in 0..sides:
    let a = corner[j]
    let b = corner[(j + 1) % sides]
```

*Checked:* `corner.cyc` gives `[0..k]Int has no .cyc`. Wanted: `for (j,
a, b) in corner.cyc.pairs()`. Proposal: `.cyc` on an array sized by an
`Int`, not only on a domain, and `pairs()` on any sequence (each element
with the next); on a `.cyc` view the last pairs with the first. Polygon
edges, circular DP and necklace problems all start with this line. What
already worked: `let corner: [sides]Corner = read()` inside a loop, with
`sides` read one line earlier. 1 of 10.

## Types and reading

### L20. A cell that may be empty, read from a 0 or a dot (2 of 10)

Today noip09p4 keeps `var board: [Line, Line]0..=9` and converts at
every use: `Digit(board[r, c])`, `board[r, c] = d.label`, `board[r, c]
== 0`. *Checked:* `[Line, Line]?Digit = read()` builds, and stops on the
first `0` with `domain value out of range`, so an optional reads as its
value and there is no way to read a `none`. `[Row, Row]?Player = read()`
for the board of cco19p1 gives `cannot read ?Player`. Wanted:

```cp
var board: [Line, Line]?Digit = read(none: 0)
let board: [Row, Col]?Player = read(none: '.')
```

Proposal: `read(none: x)` for any `?T`: the token `x` reads as `none`,
anything else as a `T`, and the generator makes both. "0 means empty"
and "-1 means none" are on the first line of many statements. 2 of 10.

### L11. Putting a member in and taking it out (noip09p4)

`Subset<Digit>` made the sudoku tables pleasant: `d not in in_row[r] |
in_col[c] | in_box[b]` is the whole test. The update is not:

```cp
in_row[r] = if on: in_row[r].with(d) else: in_row[r].without(d)
```

*Checked:* `Subset<Digit>` has no `add` (`no method add`). Wanted:
`in_row[r].add(d)`, `.remove(d)`, `.toggle(d)`. Proposal: the three
mutating methods beside `with` / `without`, as `List` has `push` beside
`+`. 1 of 10.

### L17. An unordered pair as a key, and a list under a new key (ccc10s4)

Today:

```cp
let wall = (min(a, b), max(a, b))
if wall not in beside: beside[wall] = []
beside[wall].push(Yard(p.label))
```

*Checked:* without the guard, `m[k].push(4)` builds and stops at run
time with `key not in map`; `m[k] += 1` on a new key works since
`54269ec`. Proposal: a method call on a missing key whose value type has
an empty value (`List`, `Set`, `Map`, `Subset`) starts from it, the same
rule as the compound assignment; and `unordered(a, b)`, the pair with
the smaller first, in the prelude (an undirected edge as a key). 1 of
10.

### L9. Kinds of letters (wossoly21s3; batch 3 G3)

Today: `'a' <= ch and ch <= 'z'`, and `place[ch + 32]` for the small
letter of a capital. *Checked:* `ch.is_lower` and `ch.lower` are `Char
has no`. Proposal: `is_lower`, `is_upper`, `is_digit`, `is_letter`,
`lower`, `upper` on `Char`, in userland; `Char` is one byte, so each is
a range test. 1 of 10. Corpus: `isupper|islower|isdigit|isalpha` or a
`- 'a'` is in 407 files (most of them the subtraction).

## Library

### L19. A spanning tree (ccc10s4)

Today, 11 lines: sort the doors, a `Dsu`, a total, a count of edges
used, and a comparison of the count with `pens` or `pens - 1` to find
out whether the tree spans. Wanted:

```cp
let inside = mst(doors.where(d => d.b != Yard(0)), over: Pen)      // ?(cost, used)
let outside = mst(doors, over: Yard)
print(min(inside?.cost, outside!.cost))
```

Proposal: `std.graph.mst(edges, over: V, by: e => e.cost) -> ?(cost: W,
used: List<E>)`, `none` when the edges do not join all of `V`. The
"does it span" check is the part that is easy to get wrong: the
original of ccc10s4 omits it and prints a forest's weight when the pens
are not joined through inner walls (on two separate triangles it prints
0; the answer is 6). 1 of 10. Corpus: `kruskal|mst` in any case is in 20
files.

### L27. The judge of an interactive problem (cco02p3; batch 5 K20, K21)

The port itself is fine: `let moved: 0..=1 = ask("F")` is the whole
protocol. Checking it took 40 lines of Python in `check.py` that hold
the hidden room, answer `F`, and compare the map. A co-located
`interactor` block is already an approved requirement; this port says
what it must be able to hold: hidden state made by a generator (the
room), a reply computed from the state and the question, a state change
(the robot moves), a limit on questions, and a final check of the
answer. With L5 and L1 the room would be ten lines of CP. 1 of 10.
Corpus: `fflush|cout.flush|<< flush` is in 31 files.

### L23, L24, L25. A game on a whole board with a real answer (cco19p1, sketch)

cco19p1 could not be built. It needs three things at once.

- L25: `Float`, `mean`, and decimals in `print` (batch 1 F15, batch 3
  G4, batch 4 B5). *Checked:* `unknown type Float`.
- L23: `@memo` keyed by an array value. *Checked:* `@memo (prototype)
  needs ... domain parameters`. A board of 13 cells with three states is
  a number below 3^13, so the general form is: any value of a finite
  type is a position in the domain of that type, and `[Cell]?Player` is
  finite when `Cell` is a constant-sized domain. Then `@memo` needs no
  special case, and the port needs no hand-written base-3 encoding (the
  original has one).
- L24: a changed copy of an array, `board.with(p: none, q: who)`.
  *Checked:* `expected an expression (at ':')`. `e with field: v` exists
  for structs in `plan/17`; this is the same for positions.

1 of 10, and not counted in any other item.

## Seen again from earlier lists

- Batch 6 S6: `while let q = at and g[q] != 'W'` gives `unknown name q`
  (ccc18s3).
- Batch 4 B11: an enum as an array dimension (probe for cco02p3).
- Batch 8 H1: `let k: 1..=n = read()` is still not checked for a single
  number. *Checked:* `3 9` is read and printed without a message.
  ioi05p1 has this line.
- Batch 3 G21: `take` on a list (`List<Int> has no method take`), and
  `sorted_desc` (batch 5 K10), both in the cco19p1 sketch.
- `INF` is an `unknown name`; ccc18s3 writes `dist[q] ?? n * m`. A
  `?Int` compared with `<` where `none` is larger than everything would
  remove the line: `dist[q] > here + cost` is the natural thing to write
  and is rejected.

## Already good

- `?Row` from `r + 1`: no port has a bounds test, and none had an
  off-by-one at the edge of a grid.
- `g[p]` with `p: (Row, Col)`, and `print` of a whole grid, built from a
  lambda if need be (`print([Row, Col] (r, c) => if open[r, c]: '.' else:
  'X')` in cco02p3).
- `[Row, Col]?Int = none` as a distance table, with `??`, `if let`, `e?`
  in an expression, and `min=` / `max=` that ignore `none`. After
  `54269ec`, ioi05p1's four-way `if let` ladder became `if let fence =
  min(across, along)`.
- `Subset<Digit>` with `|` and `in` (noip09p4).
- Generic fns over array dimensions: `fn above<R, C>(garden: [R, C]Int)`
  was called with rows and columns swapped and needed no change.
- `[n](from: Int, to: Int, way: 1..=4) = read()` and `let start: Cell =
  read()`: records and pairs straight from the input.
- `for x in xs` with a push to `xs` in the body is a compile error with
  a hint.
- `ask("F")` with a typed reply.

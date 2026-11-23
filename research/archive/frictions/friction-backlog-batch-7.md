# Friction backlog, batch 7: data structures and offline queries

A Stage-1 scout pass over ten hard data-structure and offline-query
submissions: ioi01p1, noi01p1, noi04p1, noi03p2, ccc14s4hard, tsoc15c2p6,
cco17p3, ioi12p3, coci20c6p5 and sac22cc5p4. The ports are in
[`proto/ports/scout_batch_7/`](../proto/ports/scout_batch_7/). Nothing in
`crates/` or `proto/cpc/` was changed. An item that an earlier backlog
already has is written up only where this batch adds something, and
otherwise listed under "Seen again".

Like batch 2, these ports use `stdlib/` and run on the reference
interpreter (`proto/cpi/`); `proto/ports/scout_batch_7/check.py` compares
each with its original on random inputs.

How to read an item. "Today" is copied from a port, so it runs on the
interpreter and gives the right answers. "Wanted" does not run anywhere
yet. The first count is exact: the ports of this batch that have the
line. The corpus count is a regular-expression search over the 710
problems with an accepted C++ submission (the latest one of each), named
in the item, so it is an estimate of how often the pattern is written and
not a list of problems that were read. The interpreter checks no types and
no borrows, so where an item says the real checker would question a line,
that is from reading `plan/19-index-and-lends.md`, not from a compiler
message.

## Suggested order for a Stage-2 sprint

1. D1, D2, D3: the three ports that came out longer than the
   library-stripped original, one missing structure each.
2. D4 and D5: nodes in a growing list. Two ports here, and the widest
   corpus count of the batch (65).
3. D8 and D9: binary search and coordinate compression, two ports each
   and 47 and 30 problems in the corpus. Both are one library function.
4. D6, D7, D10, D12: one port each, one library piece each.
5. D11 and the interpreter gaps: input shapes. Five of the ten ports
   dispatch on a command by comparing a character or a number.

## D1. No ordered bag, and none with ranks (noi04p1; batch 2 B1)

noi04p1 keeps salaries under global raises and cuts and asks for the k-th
highest. The original is 20 lines around an order-statistics tree:
`insert`, `find_by_order`, and `erase(begin())` while the lowest is under
the floor. `stdlib/` has no ordered container, so the port reads the
commands twice. The first pass exists only to learn which salaries will
occur, so that they can be numbered:

```cp
var raise = 0
var seen = Set<Int>()
for c in Cmd:
    let (op, x) = cmds[c]
    if op == 'A': raise += x
    elif op == 'S': raise -= x
    elif op == 'I' and x >= floor: seen.add(x - raise)

let bases = sorted(seen)
domain Slot = 0..bases.len
var slot = Map<Int, Slot>()
for s in Slot: slot[bases[s.to_idx]] = s
```

and the second pass counts staff per slot in a `Fenwick<Slot, Int>`, with
the size kept by hand next to it:

```cp
while size > 0:
    let lowest = staff.lower_bound(1)!.left!
    if bases[lowest.to_idx] + raise >= floor: break
    let leaving = staff.fold(lowest..=lowest)
    staff.add(lowest, -leaving)
    size -= leaving
    left += leaving
```

Wanted, in one pass and about 15 lines for the whole program:

```cp
var staff = SortedBag<Int>()
...
    Hire(x) if x >= floor: staff.insert(x - raise)
    Cut(x):
        raise -= x
        while let low = staff.first if low + raise < floor:
            staff.pop_first()
            left += 1
    Find(k): print(if let base = staff.nth_last(k): base + raise else: -1)
```

Batch 2 asked for `lower_bound` and `erase` on a bag. This batch adds two
requirements. Ranks: `nth`, `nth_last` and `rank(x)` in O(log n), which
rules out a plain B-tree without subtree sizes. And an order that is a
value: the dropped jloi16p5 keeps circles in a map ordered by where each
crosses the sweep line, which changes as the line moves, so
`SortedBag<T>(order: Order<T>)` has to accept an order that reads a `var`.

A smaller piece of the same item: `Fenwick.lower_bound` returns the
boundary where the prefix reaches the target, so the k-th element is
`staff.lower_bound(k)!.left!`, two unwraps for one idea. Wanted
`staff.kth(k) -> ?D`.

Counts: 1 port, 18 of its 42 lines. 16 problems in the corpus call
`find_by_order` or `order_of_key`; batch 2 counted the `multiset` and
`set::lower_bound` users separately.

## D2. A Fenwick tree over two domains (ioi01p1)

A Fenwick tree needs a `Group`, and a Fenwick tree is not one, so
`Fenwick<X, Fenwick<Y, Int>>` cannot be written. The port stores the inner
trees in a list and walks the outer tree itself, which brings back the two
things the library was hiding: the `+ 1` and the `p & -p`.

```cp
var slots = [Fenwick<Y, Int>(Sum<Int>) for _ in 0..=n]

fn add(x: X, y: Y, a: Int):
    var p = x.to_idx + 1
    while p <= n:
        slots[p].add(y, a)
        p += p & -p

fn left_of(x: Bounds<X>, ys: Range<Y>) -> Int:
    var (p, total) = (x.ord, 0)
    while p > 0:
        total += slots[p].fold(ys)
        p -= p & -p
    total
...
print(left_of(r.r, b..=t) - left_of(l.l, b..=t))
```

Wanted: the index of a structure may be a pair of domains, as it already
may for an array (`[X, Y]Int`).

```cp
var grid = Fenwick<(X, Y), Int>(Sum<Int>)
grid.add((x, y), a)
print(grid.fold(l..=r, b..=t))
```

The general form is that `Fenwick<D, T>` asks `D` for "the slots that
cover position p" and "the slots that make up a prefix", and a pair of
domains answers both by nesting. The same question decides whether
`SegTree<(X, Y), T>` and a 2-D `prefix` come for free.

Counts: 1 port, 12 of its 27 lines. 10 problems in the corpus have a
two-dimensional Fenwick tree (`bit[i][j]`, `update2`, `psum2`).

## D3. A sequence that splices (noi03p2)

The editor inserts and deletes in the middle of a text. A `List` has
`push` and `pop` and nothing in the middle, so every edit rebuilds the
list of pieces out of three slices:

```cp
pieces = pieces[0..i] + [s[0..p - before], s[p - before..s.len]] + pieces[i + 1..pieces.len]
...
pieces = pieces[0..i] + [text] + pieces[i..pieces.len]
...
pieces = pieces[0..i] + pieces[j..pieces.len]
```

Wanted, first, one method on `List` that says what the three lines mean:

```cp
pieces.splice(i..=i, [s[..p - before], s[p - before..]])
pieces.splice(i..i, [text])
pieces.splice(i..j, [])
```

and second, the structure the original used. With a `Seq<Char>` (a rope:
`insert(at, xs)`, `erase(range)`, `slice(range)`, all O(log n)) the port
has no pieces and no `cut` at all, and the editor is the command loop.
The piece table is fine here only because the statement limits edits to
4000; the same problem with 10^5 edits needs the rope.

Counts: 1 port, three splices and the 9-line `cut`. 1 problem in the
corpus uses `rope<>`; the implicit treaps among the 65 node-arena
problems of D4 are the same structure written by hand.

## D4. Nodes in a growing list are named by `Int` (cco17p3, ioi12p3, ioi01p1)

A domain is fixed when it is declared. A trie, a persistent tree and an
implicit tree all grow one node at a time, so their nodes cannot be a
domain, and the typed indices that make the rest of the library safe are
not available where pointer mistakes are most likely. cco17p3:

```cp
struct Node:
    kid: List<?Int>     // by the next bit
    total: Int
    ys: ?Int

var nodes: List<Node> = []
fn fresh() -> Int:
    nodes.push(Node([none, none], 0, none))
    nodes.len - 1
...
        o = nodes[o].kid[rest % 2]!
```

Here `ys` is a root in the same list, `o` is a node, `rest % 2` is a bit
and `x` is a coordinate: four meanings of `Int` in one function, and
`nodes[x]` would run. ioi12p3 goes the other way and declares the domain
too large, then counts by hand how much of it is in use:

```cp
domain Text = 0..=n                 // Text.first is the empty text
var texts = 0                       // texts made so far, not counting the empty one
...
let (old, now) = (history.last, Text.nth(texts + 1))
texts += 1
```

Wanted: a growing container that hands out its own sealed index type, the
way `scc` hands out a fresh domain.

```cp
var nodes = Arena<Node>()
let root = nodes.push(Node())           // an Arena<Node>.Id, not an Int
struct Node:
    kid: [Bit]?Id
    total: Int
    ys: ?Id
...
        o = nodes[o].kid[rest % 2] ?? return
```

An `Id` is only ever made by `push`, so it is always in range and needs no
bounds check, and it cannot be mixed with a count or a coordinate. A side
table over the same ids (`len`, `up` and `jump` in ioi12p3) is
`[nodes.Id]T` that grows with the arena. In `plan/19-index-and-lends.md`
terms this is a sealed `Index` whose bound is the arena's current length:
the open question is that the bound only grows, so an `Id` stays valid but
the type is not a frozen `let K = n`.

Persistence needs nothing more: a path copy is `nodes.push(nodes[o])`, and
a version is an `Id`.

Counts: 2 ports and the slot list of ioi01p1. 65 problems in the corpus
keep nodes in an array with integer children (`lc`/`rc`, `heap[]`,
`ch[][2]`), 7 of them with a list of persistent roots.

## D5. "Make it if it is missing" on an optional slot (cco17p3)

```cp
if nodes[o].kid[rest % 2] == none: nodes[o].kid[rest % 2] = fresh()
o = nodes[o].kid[rest % 2]!
...
if nodes[col].ys == none: nodes[col].ys = fresh()
nodes[reach(nodes[col].ys!, y)].total += v
```

The place is written three times and unwrapped with `!` although the line
above just filled it. Wanted: `??=`, which is `x f= y` from the
ergonomics audit with `??` as the operation, and has the value of the
place afterwards:

```cp
o = nodes[o].kid[rest % 2] ??= fresh()
...
let ys = nodes[col].ys ??= fresh()
nodes[reach(ys, y)].total += v
```

The same line serves a map with a default (`groups[k] ??= []`).

Counts: 1 port, 2 sites. Every node-arena problem of D4 that creates
children on demand has it (`if (!o) o = alloc()`), most of the 65.

## D6. A union-find that knows the difference between two members (noi01p1)

`Dsu<V>` answers "same group?". Food Chain needs "what is kind[x] -
kind[y]?", so the port is a union-find from scratch, 13 lines, and the
library's `Dsu` is not used at all:

```cp
fn root(a: Animal) -> (Animal, Int):
    if up[a] == a: return (a, 0)
    let (r, d) = root(up[a])
    up[a] = r
    ahead[a] = (ahead[a] + d) % 3
    (r, ahead[a])

fn relate(x: Animal, y: Animal, d: Int) -> Bool:
    let (rx, dx) = root(x)
    let (ry, dy) = root(y)
    if rx == ry: return (dx - dy) % 3 == d
    up[rx] = ry
    ahead[rx] = (d + dy - dx) % 3
    true
```

Wanted: the same structure with a `Group` for the differences, as
`Fenwick` already takes one.

```cp
var kinds = Dsu<Animal, Mod<3>>(Sum<Mod<3>>)
...
    if not kinds.relate(a, b, kind - 1): lies += 1      // false when it contradicts what is known
```

`relate(x, y, d) -> Bool` learns or checks `val[x] - val[y] == d`, and
`diff(x, y) -> ?T` reads it. With `Xor` it is the bipartite check, with
`Sum<Int>` it is distances along a line. Euclidean `%` already helped:
the original needs a `mod3` function for negative differences and the
port does not.

Counts: 1 port, 13 of 23 lines. The search found 1 weighted union-find
in the corpus by name; parity union-finds written as two nodes per vertex
are not counted.

## D7. Mo's algorithm is all driver (coci20c6p5)

What belongs to the problem is `enter`, `leave` and reading `h`. The rest
of the port is the algorithm, the same in every problem that uses it, and
it is where the window drops from `Paper` to offsets:

```cp
let block = max(1, isqrt(n))
fn mo_key(a: Ask) -> (Int, Int):
    let (b, r) = (asks[a].l.to_idx / block, asks[a].r.to_idx)
    (b, if b % 2 == 0: r else: -r)

var answer: [Ask]Int = 0
var (lo, hi) = (0, 0)               // the window is the papers at offsets lo..hi
for a in sorted(Ask, by: mo_key):
    let (l, r) = (asks[a].l.to_idx, asks[a].r.to_idx + 1)
    while hi < r:
        enter(Paper.nth(hi))
        hi += 1
    while lo > l:
        lo -= 1
        enter(Paper.nth(lo))
    while hi > r:
        hi -= 1
        leave(Paper.nth(hi))
    while lo < l:
        leave(Paper.nth(lo))
        lo += 1
    answer[a] = h
```

The order of the four loops matters (grow before shrinking), and nothing
checks it. Wanted, in `stdlib/`:

```cp
let asks: [Ask]Range<Paper> = read()
var answer: [Ask]Int = 0
for a in mo(asks, enter, leave): answer[a] = h
```

`mo` is a generator over the questions that calls `enter` and `leave`
between yields. It can be written in the language today; it needs
callbacks that change the caller's variables, which is the `inline`
callback rule of plan 19 §5.3.

Counts: 1 port, 21 of 44 lines. A handful in the corpus (the block-order
search finds 4, two of them Mo's), so this is cheap to add and rarely
used.

## D8. Binary search over a sorted list (sac22cc5p4, ccc14s4hard; batch 2 B1)

```cp
var (lo, hi) = (0, runs.len)        // runs[..lo] start at or before l, runs[hi..] after it
while lo < hi:
    let mid = (lo + hi) / 2
    if runs[mid].l <= l: lo = mid + 1
    else: hi = mid
print(if lo > 0 and r <= runs[lo - 1].r: "Y" else: "N")
```

The same six lines are in `tinted` of ccc14s4hard, and batch 2 had them in
noip18p3. The comment is there because the loop does not say which side
`lo` ends on. Wanted: one function that takes the property and returns
the split, and an optional for "the last one that has it":

```cp
print(if let run = runs.last_where(run => run.l <= l) if r <= run.r: "Y" else: "N")
...
let dark = rows.partition_point(row => row.tint + pending[b] >= need)
```

`partition_point(ok) -> Int` (how many leading elements satisfy `ok`),
`last_where(ok) -> ?T` and `first_where(ok) -> ?T` on any list or range,
with the same names on `0..n` for searching an answer.

Counts: 2 ports. 47 problems in the corpus have the hand loop and 45 call
`lower_bound` or `upper_bound` on a sorted vector.

## D9. Coordinate compression into a domain (ccc14s4hard, noi04p1)

Both ports sort the distinct values, declare a domain of that size, and
fill a map back from value to position. Seven lines in each, and the map
is keyed by `Int` in one and holds an `Int` in the other because the
typed version was more to write:

```cp
let ys = sorted(Set([panes[p].y1 for p in Pane] + [panes[p].y2 for p in Pane]))
domain Strip = 0..ys.len - 1        // strip s lies between ys[s] and ys[s + 1]
var first_strip = Map<Int, Int>()
for i in 0..ys.len: first_strip[ys[i]] = i
...
height += ys[s.to_idx + 1] - ys[s.to_idx]
```

Wanted: a function that returns a fresh domain, as `scc` does.

```cp
let (Y, ys) = compress(panes.map(p => [p.y1, p.y2]).flat)   // ys: [Y]Int, sorted and distinct
let y1 = Y.of(pane.y1)                                       // the position of a value that is present
```

ccc14s4hard then wants the gaps between neighbours as their own domain
(`Strip`, one fewer than `Y`, with `strip.lo` and `strip.hi` in `Y`), so
that `ys[s.hi] - ys[s.lo]` replaces `ys[s.to_idx + 1] - ys[s.to_idx]`.
`Bounds<D>` already relates a domain to its boundaries; this is the same
relation read the other way.

Counts: 2 ports. 30 problems in the corpus sort and `unique` a vector of
coordinates.

## D10. A domain cut into blocks, and the overlap of two ranges (ccc14s4hard)

```cp
let size = max(1, isqrt(Strip.K))
domain Block = 0..(Strip.K + size - 1) / size
fn strips(b: Block) -> Range<Strip> = Strip.nth(b.to_idx * size)..=Strip.nth(min(Strip.K, (b.to_idx + 1) * size) - 1)
...
fn add(l: Int, r: Int, t: Int):
    for b in Block:
        let (from, to) = (max(l, strips(b).first!.to_idx), min(r, strips(b).last!.to_idx + 1))
        if to - from == strips(b).len:
            pending[b] += t
        elif from < to:
            for s in from..to: tint[Strip.nth(s)] += t
            rebuild(b)
```

Every line converts to offsets and back, and the ceiling division and the
last short block are exactly the arithmetic a domain is meant to remove.
Wanted:

```cp
let (Block, strips) = Strip.chunks(isqrt(Strip.K))      // strips: [Block]Range<Strip>
fn add(r: Range<Strip>, t: Int):
    for b in Block:
        let part = r & strips[b]
        if part == strips[b]: pending[b] += t
        elif not part.is_empty:
            for s in part: tint[s] += t
            rebuild(b)
```

Two pieces: `D.chunks(k)` returning a fresh domain of blocks, and `&` on
two ranges of one domain. The second is also what the Mo's window of D7
and every "clip the query to this node" line of a hand-written tree need.

Counts: 1 port, 11 lines. 25 problems in the corpus have a block-size
constant.

## D11. Input that is not a table (noi03p2, noi01p1, ioi12p3)

Three shapes that `let x: T = read()` does not cover.

Raw text. The editor's `Insert n` is followed by n characters that may
include spaces and are split over lines. The port can only read words:

```cp
var text = ""
while text.len < n:
    let word: Str = read()
    text += word
```

so it is wrong on text with spaces, and `check.py` generates none. Wanted
`let text = read_chars(n, skip: '\n')` and `read_line()`. 4 problems in
the corpus call `getline`.

A value that may be outside its domain. Food Chain counts a statement
about animal `n + 1` as false. Reading it as an `Animal` would stop the
program, so the port reads `Int` and converts:

```cp
let (kind, x, y): (Int, Int, Int) = read()
let (a, b) = (Animal.try(x), Animal.try(y))
if a == none or b == none or not relate(a!, b!, kind - 1): lies += 1
```

Wanted: reading at an optional type does the check, and `if let` takes
apart a tuple of optionals:

```cp
let (kind, x, y): (Int, ?Animal, ?Animal) = read()
if let (a, b) = (x, y) if relate(a, b, kind - 1): continue
lies += 1
```

`read()` into a place. `last[now] = read()` is rejected: the type is
known from the array but `read` is only accepted on a typed `let`, so the
port names a temporary (`let letter: Char = read()`).

Counts: 3 ports, one shape each.

## D12. Range add under a minimum is not stocked (tsoc15c2p6; batch 2 B4)

```cp
let add = Action<Int, Int>(id: 0, compose: (f, g) => f + g, apply: (f, x) => if x == Int.MAX: x else: x + f)
var row = LazySegTree<Soldier, Int, Int>.from(Min<Int>, add, health)
```

`stdlib/lazy.cp` has `AddMax` and `AddSum`, and the `Int.MAX` guard is
the part nobody remembers. Batch 2 asked for the common updates by name;
this is the third port to spell one out. Wanted `AddMin`, or better
`LazySegTree.range_add(Min<Int>, health)` which picks the guard from the
monoid's identity. The rest of the port is what the library is for: 11
lines against 60.

Counts: 1 port, 1 line. 33 problems in the corpus have a lazy tree.

## Smaller notes

- **The last element as a place.** `runs[runs.len - 1].r = max(runs.last.r, r)`
  in sac22cc5p4: `runs.last` reads but cannot be assigned. Wanted
  `runs.last.r max= r`.
- **Records over a domain.** ioi12p3 keeps `len`, `last`, `up` and `jump`
  as four arrays because `[Text]Node` needs a value for every slot up
  front. With the arena of D4 it is one `nodes.push(Node(...))`.
- **Paired events.** ccc14s4hard pushes `(x1, .., t)` and `(x2, .., -t)`
  for every pane and sorts. 21 problems in the corpus build an `events`
  list. A `sweep(intervals)` generator that yields "enters" and "leaves"
  in order would remove the sign convention.
- **Joining strings.** noi03p2 builds its output with `shown += s[..]` in
  a loop; `"".join(parts)` or `sum` over `Str` is not there.
- **Merging intervals.** The eight-line loop of sac22cc5p4 is a library
  function (`merge_runs(segs, touching: 1)`), and with it the port is the
  query loop.

## Seen again

- **Commands** (`enum` with tags; batch 2 gaps, batch 1 F21). Five of the
  ten ports dispatch by comparing: `if op == 1` (ioi01p1, noi01p1),
  `if op == 'A'` (noi04p1, ioi12p3), `if cmd == "Prev"` (noi03p2). In
  noi04p1 the same chain is written twice, once per pass. ioi01p1 also
  has commands with different shapes and a terminator, so its loop is
  `while true` with a `break`. 27 problems in the corpus compare a
  character or word and 33 a number.
- **A whole-domain range.** `row.fold(Soldier.all)` exists and reads
  well; `LazySegTree` has no `.all` as `SegTree` does.
- **`sorted(D, by:)`** over a domain is used in three ports and is right
  every time.

## Gaps in the reference interpreter

Not language friction: the language has the construct and `proto/cpi/`
does not.

- **`else if`.** "expected ':'". The ports write `elif`.
- **An open slice.** `pieces[i..]` stops with "cannot build a range from
  1 to none"; the ports write `pieces[i..pieces.len]`. This is the
  open-ended range the ergonomics audit introduced.
- **`enumerate` and `zip`.** Unknown names. ccc14s4hard loops over
  `0..ys.len`.
- **`if let` on a tuple of optionals.** `if let (a, b) = (A.try(x),
  A.try(y))` binds the optionals themselves and the body fails with
  "expected a Animal, got none".
- **Filling an array with a domain value.** `var up: [Text]Text =
  Text.first` makes one value, not an array; `= 0` and `= ' '` do fill.
  The ports write `[Text] t => t`.
- **`read()` outside a typed `let`**, and **input by tokens only**: see
  D11.
- **`enum` and `match`**: as in batch 2.

## Already good

- `LazySegTree` with ranges of a domain: tsoc15c2p6 is the statement
  plus four lines, and `row.apply(l..=r, -x)` has no off-by-one to get
  wrong.
- Boundaries in a prefix query: `left_of(r.r, ..) - left_of(l.l, ..)` in
  ioi01p1 is "up to and including r, minus everything before l", where
  the original has `x2 + 1` and `x1`.
- Euclidean `%`: noi01p1 has no fix-up for negative differences.
- `Animal.try(x)` gives an optional instead of a trap, so "this animal
  does not exist" is a value the program can test.
- `sorted(Ask, by: mo_key)` with a tuple key replaces a struct with a
  comparison operator.
- Generators: `along(root, x)` in cco17p3 turns two recursive query
  functions into two nested `for` loops.

## Solutions that changed shape

- **ioi12p3**: a persistent segment tree with a root per command became
  the tree of texts with one jump pointer per node; no node is ever
  copied.
- **coci20c6p5**: a Fenwick tree and a binary search per question became
  two counters updated in O(1) per step of the window.
- **sac22cc5p4**: coordinate compression, a difference array and a list
  of uncovered points became merged runs and one binary search.
- **noi03p2**: a rope became a piece table.
- **noi04p1** changed shape the wrong way: an online order-statistics
  tree became two passes (D1).
- **cco17p3** kept its shape, on purpose. A `Map<(Int, Int), Int>` of
  strokes with 60 by 60 lookups per cell is eight lines, but every lookup
  is a hash where the trie stops at the first missing node.

## Two originals that are not safe oracles

Both are accepted on the judge.

- **jloi16p5** (`5445696.cpp`) gives a wrong answer when two circles sit
  side by side inside a third: for `0 0 10`, `0 -5 2`, `0 5 2` it prints
  100 and the answer is 92. It was dropped from the batch.
- **cco17p3** (`4843963.cpp`) reserves 110 trie nodes per stroke and
  keeps references into the vector while it grows. A stroke near 10^18
  can need 119 nodes; on such an input with few strokes it reads freed
  memory and aborts. `check.py` runs it only up to 10^12 and uses a brute
  force above that.

A differential test against the corpus should not assume an accepted
submission is right outside the judge's data.

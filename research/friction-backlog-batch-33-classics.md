# Friction backlog, batch 33: hard CCC / CCO / COCI / IOI classics

Ten mixed DP, tree, graph and data-structure submissions were re-solved
in [`proto/ports/scout_batch_33/`](../proto/ports/scout_batch_33/) against
the locked v2 sheet (`docs/one-page-cheat-sheet.md`, `3723dca`). Each port
runs on the reference interpreter and agrees with its original on 200
random small and medium inputs. Three more ports (ceoi22p5, coci21c1p3, cco22p3) were written and passed
the same check, then dropped because batches 32, 35 and 30 landed with
those problems meanwhile; what they showed is kept below and marked
"dropped port", since the friction is real either way. They were replaced
by ioi09p5, cco05p6 and btoi16p1. This file lists what was awkward, with
the code that had to be written and the code we wanted to write. Items
are `C1`...; "sheet gap" means v2 already promises the feature and the
interpreter prelude lacks it, "design" means v2 itself has no good answer.

## Size

Lines are non-blank, non-comment lines of the port. "Raw" is every
non-blank line of the original. "Stripped" removes what a perfect C++
contest library would already hold: the debug template, I/O setup, and
library-grade structures (the `FastCin` reader in btoi15p2, the generic
segment tree in the dropped ceoi22p5, the hash functor in cco21p2, the helper
templates and bundled grader `main` in ioi14p5).

The ten ports are 244 lines against 953 raw (74% fewer) and 549 stripped
(56% fewer). Per port, as port / raw / stripped: ccc20s4 17 / 162 / 133,
btoi18p2 23 / 80 / 52, btoi15p2 24 / 107 / 57, ioi14p5 20 / 123 / 22,
ioi12p6 38 / 88 / 54, cco20p4 29 / 77 / 50, cco21p2 31 / 92 / 54,
ioi09p5 25 / 77 / 44, cco05p6 17 / 65 / 33, btoi16p1 20 / 82 / 50. The
three dropped ports were 29 / 109 / 53 (ceoi22p5), 82 / 179 / 144
(coci21c1p3) and 50 / 98 / 69 (cco22p3).

Two numbers need a caveat. ccc20s4 is short because the port uses a
simpler algorithm (prefix counts per rotation) than the original's
incremental update, not because of the language. ioi14p5 gains nothing
over the stripped original: a twelve-line loop over two arrays is already
minimal in C++. The honest language wins are in the middle: `?Op` links in
btoi15p2, `bfs` taking a closure in btoi16p1, and, in the dropped ports,
the monoid handed to `SegTree` and a record as a `Map` key.

## Design friction

### C10. Arithmetic on `?Int` has no vocabulary

v2 says "use `?T`, not sentinels", and then offers nothing for the three
things a DP does with a cost that may not exist: take the smaller, add
two, add a number. Every port that followed the rule wrote the same
helpers; btoi16p1 writes `best = min(best ?? pay, pay)` for the same
reason. From the dropped coci21c1p3 and ceoi22p5:

```cp
fn least(a: ?Int, b: ?Int) -> ?Int:
    let x = a else: return b
    let y = b else: return a
    min(x, y)
fn add(a: ?Int, b: ?Int) -> ?Int:
    let x = a else: return none
    let y = b else: return none
    x + y
```

The interpreter also rejects the forms one tries first: `nx[c] min= d`
on a `none` cell stops with "'<' not supported", and `min(a[c] for c in
C)` over a `[C]?Int` does not skip `none`. What we wanted to write, in
the cycle DP of coci21c1p3:

```cp
g[y, z] min= f[x, y] + cost(i, z)       // none + anything is none; min= ignores none
```

Proposal: define `min=`/`max=` on `?T` targets with `none` as "no value
yet", let `+` lift over `?Int` (`none` absorbs), and make `min`/`max` of
an iterator of `?T` skip `none`. That is one rule ("`none` is the
identity of min/max and absorbs under arithmetic") instead of two helper
functions per file.

### C11. Multi-feature collision: `?Int` costs x "sum of all but one"

The tree DP in the dropped coci21c1p3 needs, for each child v, the sum of the other
children's costs plus v's alternative cost. With a sentinel the original
writes `tot - dp(v, false) + dp(v, true)` and lets a huge number cancel.
With `?Int` there is nothing to subtract when one child is impossible, and
the first draft of the port was wrong: it gave up as soon as any child
had no unpicked solution, even when that child was the one to be picked.
The fix counts the impossible children by hand:

```cp
var (total, stuck) = (if me: 1 else: 0, 0)
for v in kids:
    if let c = hang[v, st(me, false)]: total += c
    else: stuck += 1
...
if stuck == 0 or (stuck == 1 and plain == none):
    hang[u, st(false, me)] = least(hang[u, st(false, me)], total - (plain ?? 0) + w)
```

The differential check caught it on the ninth input. This is the pattern
where `?T` is strictly harder to get right than a sentinel, and it is a
standard one ("exactly one child differs"). Proposal: a stdlib helper for
it, for example `swap_one(kids, base: v => ..., alt: v => ...) -> ?Int`,
which returns the best total with exactly one element taking `alt`, or
prefix/suffix folds over `opt(Sum)` so "all but v" is a fold, not a
subtraction.

### C8. `Bool` is not an index type, so flag-indexed tables are packed by hand

Two of the dropped ports have DP states with boolean components. cco22p3 could use a
record in a `Map`; coci21c1p3 needs a dense table and could not write
`[V, Bool, Bool]?Int`:

```cp
domain St = 0..4                            // picked + 2 * covered
fn picked(s: St) -> Bool = s.to_idx % 2 == 1
fn covered(s: St) -> Bool = s.to_idx >= 2
fn st(covered: Bool, picked: Bool) -> St = St((if picked: 1 else: 0) + (if covered: 2 else: 0))
var hang: [V, St]?Int = none
```

Five lines of bit packing is exactly the C++ `x & 1`, `x & 2` we set out
to remove. Wanted: `Bool` (and any two-value enum) as a domain, and a
record of domains as a product index:

```cp
type St = (covered: Bool, picked: Bool)
var hang: [V, St]?Int = none
for a in St: ...                            // four states
```

Related: the `if flag: 1 else: 0` shape appears seven times in the batch.
`Bool.to_int`, or `count(...)` over a tuple of flags, would remove it.

### C12. Multi-feature collision: boundaries x two pointers

btoi18p2 keeps a left boundary and a right index. The sheet's boundary
model is sound, but the moving-left-edge loop needs three different
conversions in four lines:

```cp
best = min(best ?? n, (lo..hi.r).len)   // `hi.r - lo` is rejected: a boundary is not a number
let gone = dna[lo.right!]
...
lo = lo.next!
```

Both `!` are provably safe (the window is non-empty) but the reader has to
check that. Wanted: a window value that owns the pair and exposes the
operations a two-pointer loop does:

```cp
var w = Pos.window()                    // empty at the left end
for hi in Pos:
    w.grow()                            // takes in hi
    ...
    while unmet == 0:
        best min= w.len
        let gone = dna[w.shrink()]      // drops and returns the leftmost index
```

`slide_until` on the sheet is stateless and yields `(lo, hi)` pairs; it
does not fit here because the body must update counts on both edges.

### C13. A node set that is "leaves, then rounds" has no type

ioi12p6 builds a tree whose nodes are the n starting positions followed
by the c rounds. The port uses one domain and tests the ordinal:

```cp
domain Node = 0..n + c                      // the leaves, then the rounds
let node = Node(n + round)
var head: [Pos]Node = [Pos] p => Node(p.to_idx)
for v in Node:
    if v.to_idx < n: ...
```

Every one of those lines is raw index arithmetic of the kind domains are
meant to prevent, and `Node(n + round)` is an off-by-n waiting to happen.
The same shape appears in Kruskal reconstruction trees and virtual
source/sink graphs. Wanted: a sum of domains with typed injections and a
match:

```cp
domain Node = Pos | Round
kids[Node(round)].push(head[first])
for v in Node:
    match v:
        Pos(p): ...
        Round(r): ...
```

### C7. No order-statistics container

ioi09p5 needs "the lowest free space" and pushes ordinals through a
`Heap<Int>` because there is no ordered set. ioi12p6 needs "the s-th knight still standing" and "remove a run of
them". The original uses a policy-based tree. The port uses a Fenwick tree of 0/1 and its descent:

```cp
var alive = Fenwick<Pos, Int>.from(Sum<Int>, [Pos] _ => 1)
let first = alive.lower_bound(s + 1)!.left!
```

The double `!` and the `s + 1` (rank to prefix sum) took two attempts;
the first draft used `.right`. The sheet lists `Set` but no `nth`. Wanted:
`Alive<Pos>` (or `Set<Pos>.nth(k)`) with `nth(k) -> ?Pos` and
`remove(p)`, implemented in userland on `Fenwick`.

### C6. Binary search over a sorted list of records

The dropped cco22p3 asks "which slide of this room started last at or before t"
three times. The port hand-writes the loop:

```cp
fn up_at(room: Room, t: Int) -> Int:
    var (lo, hi) = (0, shows[room].len)
    while lo < hi:
        let mid = (lo + hi) / 2
        if shows[room][mid].l <= t: lo = mid + 1
        else: hi = mid
    lo - 1
```

and returns `-1` for "none", because the caller then does `j += 1` and
compares positions; a `?Int` here made the three callers longer. Sheet
gap in part (`bsearch_first`, `first_ge` are not in the prelude), design
in part: the sheet's `first_ge` is defined on `compress` results, not on
a `List<T>` with a key. Wanted: `shows[room].last_le(t, by: s => s.l)`
returning a boundary, so "the one after it" is `.right` and "none" is the
left end, with no `-1`.

### C5. Circular ranges

ccc20s4 counts letters in a block of seats that may wrap around the
table. The port doubles the table and builds three prefix lists by hand.
The sheet has `prefix(a)` and `v.cyc(k)`, but nothing that makes a
wrapping range a value. Wanted:

```cp
let before = [Group] g => prefix(Seat, i => s[i] == g.letter)
before[x].sum(at.cyc_range(size(x)))        // a range that may wrap
```

### C3. Printing and rotating lists

cco20p4 prints a walk rotated to start at each city. `print(list)`
writes `[1, 2, 1, 3]`, there is no join, and no `find` or `rotate_to`, so
the port carries an offset and prints one number per line (cco05p6
likewise shifts a list by hand for want of `remove_at`):

```cp
while walk[start % walk.len] != c: start += 1
start %= walk.len
for i in 0..walk.len: print(walk[(start + i) % walk.len].to_idx + 1)
```

The sheet has `rotate_left(mut a, k)`. Wanted in addition: `print(xs)`
writing a space-separated line (the one output format every judge uses),
`xs.find(x) -> ?Int`, and domain values printing in their declared base
so `.to_idx + 1` goes away when the domain is `1..=n`.

## Sheet gaps (v2 promises it, the interpreter lacks it)

- **C1.** `count`, `all`, `any` are not in the prelude. btoi18p2 writes
  `sum(1 for s in Sym if need[s] > 0)`; cco20p4 writes
  `sum(1 for c in City if seen[c]) == n` for `all(seen)`.
- **C2.** `if let` and `else:` unwrapping are statements only.
  `print(if let t = below(i, 1): ops[t] else: 0)` and
  `let (p, q) = (a else: return b, b else: return a)` are parse errors;
  each costs an extra binding.
- **C4.** `HashMap` does not exist; cco21p2 uses the ordered `Map`.
  `var via: HashMap<Int, Int> = []` silently makes a list and fails later
  with "index 5 is out of bounds for a list of length 0".
- **C9.** `var when: [V]Int = n` does not fill: the fill form only takes
  a literal, and the failure is "cannot index 22" at the first read.
  `= 0` and `= none` work, so this one is a trap.
- **C14.** `read()` only works as the initializer of a typed `let`.
  `joins.push(read())` is "unknown name `read`", and a `let` cannot name
  record fields, so ioi14p5 reads a tuple and rebuilds the record:

  ```cp
  let j: (P, Int) = read()
  joins.push((host: j.0, how: j.1))
  ```

  Wanted: `let joins: [1..n](host: P, how: Int) = read()`.
- **C15.** No `loop:`; cco21p2 writes `while true:`.
- **C16.** No `prefix` (ccc20s4), no `bsearch_first` (dropped cco22p3).

## What worked without friction

`SegTree<Who, Block>.new(Monoid<Block>(id, join))` in the dropped ceoi22p5 took the
custom four-field node with no ceremony and replaced a 28-line struct.
`[Lg, Op]?Op` with `for j in Lg.rev` and `if let v = up[j, u]` reads
better than the `-1` table in btoi15p2 and removed the original's
`assert`. A record as a `Map` key and heap element in the dropped cco22p3 replaced a
3-D array plus a 4-tuple. `P.rev`, tuple assignment of two cells at once
in ioi14p5, and `s.room.cyc(1)` for "the other room" were all first-try.

## Suggested order

C10 first: it is the most frequent, and C11 is easier to solve once
`none` has arithmetic. Then C8 (Bool and record indices), which removes
packing code from every flag DP. C13 and C12 are new abstractions and
should go through a bakeoff. The sheet gaps C1, C2, C9, C14 are small
interpreter work and each one bit more than one port.

### C17. `let ... else:` with a multi-line body inside a loop

btoi16p1 skips a root that cannot reach everyone. The natural form is a
labelled `continue`; the port sets a flag value instead:

```cp
for p in P:
    let d = depth[p] else:
        pay = -1
        break
    pay += d
if pay >= 0: best = min(best ?? pay, pay)
```

Wanted: `let total = sum?(depth[p] for p in P) else: continue`, a sum
that is `none` when any term is. That is the C10 rule again (`none`
absorbs under `+`), applied to `sum`.

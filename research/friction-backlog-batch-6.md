# Friction backlog, batch 6: greedy, sorting, two pointers and sweeps

A Stage-1 scout pass over ten greedy, sorting, two-pointer, sweep-line and
coordinate-compression submissions. The ports are in
[`proto/ports/scout_batch_6/`](../proto/ports/scout_batch_6/): ccc21s3,
coci11c4p4, cco17p5, nccc8s3, dmopc20c4p3, joi16fp2, dmopc21c8p1, cco04p2,
etsr23r1p2 and dmopc20c7p1. Each is there twice: the form we want, with
`// WISH Sn` on the lines the prototype rejects, and `today/`, the form
that builds. This file follows the format of
[`friction-backlog.md`](friction-backlog.md) (batch 1). Its items are
numbered `S1`... so they do not collide with `F` (batch 1), `G` (batch 3),
`B` (batches 2 and 4), `D` (batch 7) or `H` (batch 8). An item an earlier
batch already has is named with its number and only what is new is added.

What was checked: all ten `today/` ports build with `proto/cpc` at commit
`5aca634` and agree with their originals on 200 random cases each
(`proto/ports/scout_batch_6/today/stress.sh`), again on larger random
cases, and on one largest-size input each. So every "today" snippet below
is a line a port really contains. Every error message quoted was produced
by a small probe with `proto/cp.py build` at that commit. The "wanted"
snippets do not build. A count such as "4 of 10" is the number of the ten
ports that contain the awkward line. A corpus count is the number of the
710 problems with an accepted C++ submission whose latest one matches a
pattern; it is a search, not a reading, so treat it as a size and not as a
measurement.

Two more problems were ported and then dropped, because batches 2 and 7
landed them while this batch was being written: cco20p2 and sac22cc5p4.
Where they showed the same friction as a port that stayed, they are named.

## Suggested order for a Stage-2 sprint

1. S1, S2, S3: a wrong answer with no message, a crash, and an argument
   that is ignored.
2. S13 and S12: the ordered lookup and the two-pointer drain. Together
   they are what greedy and sweep code is made of, and S13 is why one port
   is longer than its original.
3. S8: reading two numbers from one line. 5 of 10, and the plan's own
   first example does not build.
4. S6 and S11: `while let ... and`, the last element as a place, and a
   max-heap. 4 and 2 of 10, small changes.
5. S15, S17, S19, S9: one library function or one parser rule each.
6. S4, S5: what the stress generator can be told.
7. S14, S16, S18, S10, S7.

## Bugs

### S1. A word read in a loop of a `@multitest` moves the rest of the test into the loop (fixed in Stage-2 sprint #2; cco04p2)

*Checked.* This program should print one number for each test:

```cp
@multitest
fn solve():
    let n: 1..=5 = read()
    var total = 0
    for _ in 0..n:
        let word: [domain W]Char = read()
        total += W.len
    print(total)
```

On `1 3 ab abc a` it prints `2`, `5`, `6`: the `print` runs inside the
loop. The same loop at the top level of a file prints `6`. The first
cco04p2 read its tiles this way and printed the rows after every tile.
Today the read is moved into a function of its own (`read_tile`).
Proposal: a domain introduced by a `read` ends with the block it is
declared in, in a function as at the top level.

*Fixed.* In a function, as at the top level, the input a loop reads ends where the outermost loop ends. Locked by `proto/tests/redteam/multitest_word_loop.cp`.

### S2. `compress` of a list expression crashes (fixed in Stage-2 sprint #2; dmopc20c4p3)

*Checked.*

```cp
let (X, col) = compress([c.x for c in climbers])
```

builds, and the program stops with `cannot create std::vector larger than
max_size()`. The emitted C++ writes the argument twice, once for `begin()`
and once for `end()`, so the two iterators belong to different lists.
Today:

```cp
let xs = [c.x for c in climbers]
let (X, col) = compress(xs)
```

Proposal: evaluate the argument once. Worth a sweep of the other builtins
that take a list: `sort`, `sorted`, `runs`, `counts`, `prefix`.

*Fixed.* `compress` evaluates its argument once. The other builtins that take a list (`sorted`, `x in [...]`, `sorted(..., by:)`) were checked with a list expression under ASan and already did. Locked by `proto/tests/redteam/list_expr_once.cp`.

### S3. `MinHeap<T>(by: ...)` takes the argument and ignores it (fixed in Stage-2 sprint #2; probe for dmopc20c7p1)

*Checked.* plan/10 writes `Heap<(Int, V)>(by: min)`, so the first thing
tried for a max-heap was:

```cp
var h = MinHeap<Int>(by: x => -x)
h.push(1)
h.push(3)
print(h.pop()!)         // prints 1
```

It builds and pops the smallest. `MaxHeap<Int>()` gives `unsupported
expression generic`. Proposal: until S11 lands, any argument to `MinHeap`
is a compile error that names the negated key as the way to get a
max-heap today.

*Fixed.* Any argument to `MinHeap<T>()` is a compile error that names the negated key (`proto/tests/bad/minheap_by.cp`). `MaxHeap` waits for S11.

## What the stress generator can be told

### S4. Bounds inside a record, and an input that is sorted (5 of 10; batch 8 H1)

The scalar half of H1 is fixed at `5aca634`: `let k: n..=3000 = read()` is
now generated and checked as written, and joi16fp2 dropped its `where k >=
n`. Three things are left.

A bound inside a record that names an earlier input is still ignored.
*Checked:* with `let people: [n](need: 0..=n, price: 1..=1e9) = read()`
the generator gave `need = 4` for `n = 1`, and the read did not object.
cco17p5 today adds `where all(p.need <= n for p in people)`, which throws
away a third of the cases at size 6 and five in six at size 25.

A field cannot be bounded by the field before it. *Checked:* `[n](l:
1..=100, r: l..=100)` gives `unknown name l`. sac22cc5p4 (dropped) wrote
`where all(s.l <= s.r for s in given)` twice and kept 30 of 200 cases
until it got two hand-written generators.

There is no way to say that an input is sorted, or that neighbours differ,
or that every value occurs. joi16fp2 needs increasing stops that start at
1 and end at `stations`, and today that is a generator of five lines
beside a `where` that still rejects half the cases:

```cp
fn random_stops() -> [E]1..=stations:
    var at: [E]1..=stations = 1
    for e in E: at[e] = rand(1, stations)
    at.sort()
    at[E.first] = 1
    at[E.last] = stations
    at
let express: [E]1..=stations = read(gen: random_stops) where all(express[e] < express[f] for e in E if let f = e + 1)
```

coci11c4p4 (neighbours differ) and nccc8s3 (every number of `1..=k`
occurs) reject 36% and 20% of their cases the same way. Proposal, in
order of value: (1) finish H1 for records, and let a field's range name
the fields before it; (2) a small set of input shapes the generator
understands and the reader checks: `read(increasing)`, `read(distinct)`,
`read(covers: V)`, the same mechanism as `read(gen: ...)` with library
generators; (3) when a `where` rejects more than half the cases, `cpl
stress` says so, because a run that reports "30 tests agree" looks like a
pass. Sorted or distinct input is declared in a large share of statements;
it was not counted.

### S5. A table of `rand` values does not fit a narrow element type (fixed in Stage-2 sprint #2; joi16fp2)

*Checked.* In the generator above, the natural first line is

```cp
var at: [E]1..=stations = [E] e => rand(1, stations)
```

It builds, and `g++` rejects the output: `could not convert ... from
'vector<long long int>' to 'vector<int>'`. `sorted([E] e => rand(1,
stations))` fails the same way. Today the table is declared with a value
and filled by a loop. Proposal: a table generator takes its element type
from the declaration, and `rand(lo, hi)` has the type `lo..=hi`.

*Fixed.* A table generator takes its element type from the declaration, and a table whose elements are stored at another width is copied across where it is assigned. Locked by `proto/tests/redteam/rand_table_narrow.cp`. `rand(lo, hi)` still has the type `Int`.

## Language and prelude

### S6. `while let ... and`, and the last element as a place (4 of 10)

A monotone stack and a two-pointer drain both say "while there is a top
and it is in the way, remove it". *Checked:* `if let x = xs.last() and x >
2` builds, and the same condition in a `while` gives `unknown name x`.
So today the top is spelled out twice, or three times:

```cp
while kept.len > 0 and kept[kept.len - 1] > x and last[kept[kept.len - 1]] > i:    // nccc8s3
    chosen[kept.pop()!] = false
while front < small.len and small[small.len - 1] + x >= m: partners.push(small.pop()!)   // dmopc21c8p1
```

and cco04p2 reads the last tile of a row as `rows[r.idx][rows[r.idx].len -
1].b`. Wanted:

```cp
while let top = kept.last and top > x and last[top] > i:
    chosen[top] = false
    kept.pop()
```

Three small things are tangled here. (1) `while let` should chain with
`and` exactly as `if let` does (decision 11, "`if let a = x and let b =
y`"); batch 4 B12 is the same gap for `if`. (2) *Checked:* a list has
`.len` as a field and `.last()` as a method (`has no .last`), and a heap
has `.len()` as a method (`has no .len`). One spelling: `len`, `first`
and `last` are fields everywhere. (3) `xs.last` is a value, so the last
element cannot be changed through it. *Checked:* `xs.last() += 1` gives ``
`+=` between ?Int and Int ``, and sac22cc5p4 (dropped) wrote
`cover[cover.len - 1].r max= s.r`. Proposal: `last` and `first` are
projections (plan/19 §3.4) that lend the element, so `cover.last!.r max=
s.r` works. Ports: nccc8s3, dmopc21c8p1, cco04p2 and the wanted form of
etsr23r1p2. In the corpus 12 problems test a stack's top inside a `while`
condition.

### S7. Leaving the program early (1 of 10, 1 dropped)

dmopc21c8p1 prints `-1` and stops as soon as a number has no partner.
*Checked:* a `return` at the top level of a file gives `return outside a
function`, although decision 6.3 says "`return` ends a test". Today the
port carries a flag through the loop:

```cp
var stuck = false
while line.len < n and not stuck:
    ...
    if let y = partners.pop(): line.push(y)
    else: stuck = true
if stuck: print(-1)
else: print(line)
```

Wanted:

```cp
let y = partners.pop() else:
    print(-1)
    return
line.push(y)
```

cco20p2 (dropped) had the same flag, `possible`. Two smaller gaps were hit
on the way: *checked*, `let x = if let c = f(): g(c) else: false` gives
`expected an expression (at 'let')`, and `.map` on an optional gives `.map
on ?T is not in the prototype yet`; and `mut fn` (decision 5.10) is a
parse error, `expected end of line (at 'fn')`, while a plain `fn` may
write the variables above it. Proposal: implement 6.3 in the prototype;
it is the most common shape of an impossible case. In the corpus 23
problems print `-1` and return on the next line.

### S8. Two numbers on one line, a mark between two numbers, and an order with a kind (6 of 10; batch 7 D11)

plan/10's second program starts `let (n, k): (1..=2e5, 0..=1e14) =
read()`. *Checked:* it gives `unknown name read`. Every port that reads
two sizes writes two lines, and joi16fp2 writes seven:

```cp
let stations: 2..=1e9 = read()
let n: 2..=3000 = read()
let k: n..=3000 = read()
let slow: 1..=1e9 = read()
let fast: 1..=1e9 = read()
let semi: 1..=1e9 = read()
let limit: 1..=1e18 = read()
```

Wanted, following the lines of the input:

```cp
let (stations, n, k): (2..=1e9, 2..=3000, n..=3000) = read()
let (slow, fast, semi): (1..=1e9, 1..=1e9, 1..=1e9) = read()
let limit: 1..=1e18 = read()
```

This is syntax that is already designed; a later item of a tuple may name
an earlier one, as `k` does. Ports: nccc8s3, dmopc20c4p3, joi16fp2,
dmopc21c8p1, etsr23r1p2.

cco04p2 reads tiles written `3-5`. Today that is a word read and a parser
of six lines, which leans on where the digits sit in ASCII (batch 3 G3):

```cp
fn read_tile() -> Tile:
    let word: [domain W]Char = read()
    var ends: List<Int> = [0]
    for c in word:
        if c == '-': ends.push(0)
        else: ends[ends.len - 1] = ends[ends.len - 1] * 10 + (c - '0')
    (min(ends[0], ends[1]), max(ends[0], ends[1]))
```

and the output is four `write` calls for each tile, because text next to a
value cannot be printed in one piece (batch 3 G4). Wanted:

```cp
let given: [n](a: Int, "-", b: Int) = read()
...
for row in rows: print(" ".join("{t.a}-{t.b}" for t in row.rev))
```

Proposal: a string literal inside a record type is a mark that `read`
skips and the generator writes. The same piece reads `12:30`, `3/4` and
`(1,2)`. Because the typed generator cannot make such a word today, this
port is the only one stressed by a script (`cco04p2_gen.py`).

etsr23r1p2 reads orders `1` or `2 l r`. Today it is a function that reads
the kind and returns a record with made-up fields for kind 1. Wanted: an
enum with tags and fields, read by its declaration, which plan/16 already
sketches ("enums read by their declared Read"):

```cp
@tags("1", "2")
enum Order:
    Count
    Add(l: 1..=n, r: l..=n)
match read::<Order>(): ...
```

This is D11 again; the tagged enum without fields is what the prototype
has.

### S9. A comprehension with two `for`s (1 of 10)

ccc21s3 wants both ends of every friend's range in one sorted list.
*Checked:*

```cp
let ends = sorted([(at, f.w) for f in friends for at in [f.p - f.d, f.p + f.d]])
```

gives `unknown name f`: the second `for` and the element cannot see the
first variable. Today:

```cp
var ends: List<(at: Int, w: Int)> = []
for f in friends:
    ends.push((f.p - f.d, f.w))
    ends.push((f.p + f.d, f.w))
```

Proposal: a comprehension nests its `for`s left to right, as its `if`s
already do. Every sweep that turns a range into two events is this shape:
here also dmopc20c4p3 (one list from two arrays, today two loops) and
etsr23r1p2, which uses the new `xs + ys`. Batch 1 F19 (`Steps`) removes
the event list altogether where the events are range adds; this item is
for the sweeps where they are not.

### S10. A table that is filled later has no honest first value (2 of 10, 1 dropped)

```cp
var last: [V]I = I.first                // nccc8s3; every cell is written by the next line
var fence: [I]Int = 0                   // coci11c4p4
var plan: [Day]Day = Day.first          // cco20p2 (dropped)
```

`I.first` is a made-up value: if a cell were missed, the program would
quietly use position 1. *Checked:* `var plan: [Day]Day` with no value
builds today and fills the table with zeroes, which for a table of
positions is the first position, and nothing in `decisions.md` says
whether that is meant. Proposal: allow the declaration without a value,
and make the meaning "not written yet": debug runs trap a read of a cell
that was never written (the same place as the overflow and index checks
of decision 7.1), and the judge build zero-fills. `[V]?I = none` stays
for the cases where "no value" is an answer. Where the fill has a name it
should be a function instead: `last` here is S15's `last_index`.

## Library

### S11. A max-heap, and a heap of the largest count (2 of 10, 1 dropped; batch 2 B5)

B5 asks for `MinHeap<T>()`, `MaxHeap<T>()` and `Heap.by(key)` in the
library. `proto/cpc` has only `MinHeap`, so both greedy ports that want
the largest negate their keys and negate them back:

```cp
var left = MinHeap<(Int, Int)>()        // (-count, -kind): the largest count is on top
for k in K: put_back(-slots[k].len, -k.idx)
...
pairs.push((K(-k1), K(-k2)))            // dmopc20c7p1
if let e = free.pop(): plan[d] = Day(-e)                // cco20p2 (dropped)
```

Each negation is a place to be wrong, and the comment is there because
the type no longer says what the heap holds. Wanted: `MaxHeap<(count:
Int, kind: K)>()`. Two additions to B5 from these ports: the heap must
take a position or a record as it is, since negating is only possible for
numbers; and `peek` is missing (`has no method min`), which a "pop while
the top is stale" loop needs with S6. In the corpus 77 problems use a
priority queue and at least 10 of them a max-heap.

### S12. A sorted list that is drained from either end (2 of 10)

This is the two-pointer loop. dmopc21c8p1 gives numbers out from the
front of a sorted list and promotes them from its back; today that is a
list, an index and three tests of `front < small.len`:

```cp
var small = sorted([x for x in a])
var front = 0
...
    var x = 0
    if front < small.len:
        x = small[front]
        front += 1
    else: x = partners.pop()!
    line.push(x)
    while front < small.len and small[small.len - 1] + x >= m: partners.push(small.pop()!)
```

Wanted:

```cp
var small = Deque(sorted(a))
while let x = small.pop_front() ?? partners.pop():
    line.push(x)
    for big in small.pop_back_while(b => b + x >= m): partners.push(big)
```

dmopc20c4p3 walks two sorted lists together. Today it avoids the second
pointer by putting sweepers and climbers into one list of records with a
`sweeps: Bool` field and sorting that (batch 1 F14 is this workaround).
Wanted:

```cp
var coming = Deque(sorted(climbers, by: c => c.y))
for s in sorted(sweepers, by: s => s.y).rev:
    for c in coming.pop_back_while(c => c.y >= s.y): standing.add(c.x)
    crossings += standing.count(..=s.x)
```

Proposal: `Deque<T>` in the library (`push_front`, `push_back`,
`pop_front`, `pop_back`, `front`, `back`), built from a list in O(1),
with `pop_front_while(ok)` and `pop_back_while(ok)` as generators that
remove and yield elements while `ok` holds. `pop_while` is the general
statement of "advance the pointer while the condition holds": the pointer,
its bound test and the off-by-one of the last element are inside it.
*Checked:* `xs.pop_front()` gives `List<Int> has no method pop_front`.
Three originals of this batch have the hand loop (`while (cr <= x)`,
`while (j < m && ...)`, `while (it != s.end() && it->first <= r)`); in
the corpus 15 problems advance a pointer in a `while` with a bound test
and 11 use a `deque`.

### S13. The first key at or after a value (2 of 10, 1 dropped; batch 7 D8, batch 2 B1)

etsr23r1p2 keeps a set of ranges that do not overlap and needs "the first
range that starts at or after l" and "the last one that starts before l".
*Checked:* `s.first_ge(2)` gives `no method first_ge on Set<Int>`, and the
same on `Map`. `first_ge` and `last_le` exist only on a 1-D array. So the
port reads every order first, compresses the coordinates, and keeps the
starts in a max-tree in which a cell holds its own number:

```cp
let ends = [o.l for o in orders] + [o.r for o in orders]
let (X, coord) = compress(ends)
var starts = SegTree<X, Int>(0, (a, b) => max(a, b))
...
while starts.query(first, last) > 0: gone += throw_out(X(starts.query(first, last) - 1))
if let before = first - 1:
    let top = starts.query(X.first, before)     // the last range that starts before l
    if top > 0 and end_of[X(top - 1)] >= o.l: gone += throw_out(X(top - 1))
```

It is 35 lines against the original's 33, the only port of the batch that
is longer than its C++, and it had to become offline. Wanted:

```cp
var held: Map<Int, Int> = []            // each range, as start -> end
while let (start, _) = held.first_ge(l) and start <= r:
    held.remove(start)
    gone += 1
if let (start, end) = held.last_lt(l) and end >= l:
    held.remove(start)
    gone += 1
held[l] = r
```

Proposal: `first_ge`, `first_gt`, `last_le` and `last_lt` on `Set` and
`Map`, returning `?K` and `?(K, V)`. They are the four `lower_bound`
shapes, named by what they find, and they match the names the arrays
already have.

cco04p2 searches a sorted list that grows. D8 asks for `partition_point`,
`first_where` and `last_where` on any list, and this port is one more
vote. Today it declares a domain inside the loop to borrow the domain's
binary search:

```cp
for t in tiles.rev:
    domain R = 0..rows.len
    if let r = R.first_where(r => rows[r.idx][rows[r.idx].len - 1].b >= t.b): rows[r.idx].push(t)
    else: rows.push([t])
```

One addition to D8: here the row that is found is written to, so
`first_where` has to give the place (an index, or a lend of the element)
and not a copy. sac22cc5p4 (dropped) built a second array of the starts
only to call `last_le` on it; with a key it is `cover.last_le(a.l, by: s
=> s.l)`. In the corpus 26 problems call `lower_bound` or `upper_bound`
on a `set` or `map`, and 14 use a `multiset` (B1, D1).

### S14. Counting in a sorted bag (1 of 10, 1 dropped; batch 1 F13, batch 2 B1, batch 7 D1)

dmopc20c4p3 counts the climbers standing at or before a point. Today it
is `compress`, a `SegTree` of counts and two lookups with `!` and `if
let`:

```cp
let xs = [c.x for c in climbers]
let (X, col) = compress(xs)
var standing = SegTree<X, Int>(0, (a, b) => a + b)
...
if let reach = col.last_le(s.x): crossings += standing.query(X.first, reach)
...
let at = col.first_ge(s.x)!
standing.set(at, standing.get(at) + 1)
```

Wanted: `standing.add(c.x)` and `standing.count(..=s.x)` on the ordered
bag that F13, B1 and D1 already ask for. Three additions. (1) An insert
is `set(at, get(at) + 1)`; a tree of sums wants `add(at, 1)`, or a lend
of the cell so that `standing[at] += 1` works. (2) *Checked:*
`t.query(I.first..)` gives `no method query on SegTree<I, Int> (set(i,
v), get(i), query(l, r))`: the open ranges of the ergonomics audit do not
reach `query`, which is where "everything after x" is asked most. cco20p2
(dropped) wrote `if let later = plan[d] + 1: moves += placed.query(later,
Day.last)`. (3) cco20p2 counted inversions with that tree; `inversions(xs)`
belongs in the library.

### S15. Grouping by a key (2 of 10, 1 dropped)

```cp
var by_need: [0..=n]List<Int> = []                              // cco17p5
for p in people: by_need[p.need].push(p.price)

let (K, name) = compress(kind)                                  // dmopc20c7p1
var slots: [K]List<Slot> = []
for s in Slot: slots[name.first_ge(kind[s])!].push(s)

var last: [V]I = I.first                                        // nccc8s3
for i in I: last[a[i]] = i
```

Wanted:

```cp
let by_need = group(people, by: p => p.need)        // [0..=n]List<...>, since the key is a range
var slots = group(Slot, by: s => kind[s])           // Map<Int, List<Slot>>, since the key is any Int
let last = last_index(a)                            // [V]I
```

Proposal: `group(xs, by: key)` in the library, returning an array of
lists when the key type is a domain or a range and a `Map` otherwise, the
same rule `Counter<K>` has; `counts` is `group` with the lists replaced by
their lengths. `first_index` and `last_index` are the two folds that keep
one element. With `group`, dmopc20c7p1 needs no `compress` at all. cco20p2
(dropped) had the same two lines (`due_on`). In the corpus 27 problems
push `i` into a list chosen by `a[i]`.

### S16. Sorting positions, and handing out in order (1 of 10; batch 3 G21)

coci11c4p4 sorts the positions of the fence by a key and gives them the
boards in ascending order. *Checked:* `sorted(I, by: ...)` gives `sorted
needs a list or 1-D array`; `by: rank` with `rank` a function gives
`expected a one-parameter lambda`; `zip` is `unknown name`. Today:

```cp
var pile = sorted([b for b in boards], by: b => -b)     // the shortest board is on top
var fence: [I]Int = 0
for i in sorted([i for i in I], by: i => rank(i)): fence[i] = pile.pop()!
```

The boards are sorted backwards only so that `pop` can stand in for
walking two lists together. Wanted:

```cp
for (i, board) in zip(sorted(I, by: rank), sorted(boards)): fence[i] = board
```

Proposal: (1) `sorted` and every other function over a sequence accept a
domain, since a domain is a sequence of its positions; (2) a function
name is accepted where a one-parameter lambda is (batch 1 F12 is the
field-name half of this); (3) `zip` (G21). "Sort the indices by a key" is
the standard first line of an offline solution; it was not counted.

### S17. Neighbours in a line (2 of 10)

Three shapes of "this element and the next" in two ports:

```cp
where all(old[i] != old[j] for i in I if let j = i + 1)         // coci11c4p4
print(sum(abs(fence[i] - fence[j]) for i in I if let j = i + 1))
for e in E:                                                     // joi16fp2
    if let f = e + 1:
        let from = express[e]
        ... express[f] ...
```

and one of "the elements on either side":

```cp
var pull = 0
for side in [i.prev, i.next]:
    if let j = side: pull += if old[i] > old[j]: 1 else: -1
```

Wanted:

```cp
where all(x != y for (x, y) in old.adjacent)
print(sum(abs(x - y) for (x, y) in fence.adjacent))
for (from, to) in express.adjacent: ...
let pull = sum(sign(old[i] - old[j]) for j in i.neighbours)
```

Proposal: `xs.adjacent` yields the pairs of neighbouring elements of any
sequence, and `I.adjacent` the pairs of positions; it is `windows(2)` of
batch 1 F6 as a pair, which is the case that is nearly always wanted.
`i.neighbours` is the open "Neighbours" item of the ergonomics audit and
`sign` is G20. *Checked:* `I.windows(2)` gives `type I has no method
windows`, and `a.pairs()` exists but means every unordered pair, which is
the wrong guess to make under time pressure. In the corpus 143 problems
index `[i + 1]`.

### S18. A set of ranges (1 of 10, 1 dropped; batch 1 F19)

etsr23r1p2 is a set of disjoint ranges with one operation, "add this
range and tell me which ranges it touched". sac22cc5p4 (dropped, now in
batch 7) is the union of ranges and "is this range inside it". Both
were written on top of something else (S13 here; a merge loop and a
binary search there), and both are short on the right type:

```cp
var held = RangeSet<Int>()
let gone = held.take_touching(l..=r).count      // the ranges that l..=r overlaps, removed
held.insert(l..=r)

let cover = union(s.l..=s.r for s in given)     // stretches with a gap between each two
print(if cover.contains(a.l..=a.r): "Y" else: "N")
```

Proposal: `RangeSet<K>` in the library, a `Map` from start to end kept
disjoint, with `insert` (merging), `take_touching`, `contains(range)`,
`covering(x) -> ?Range<K>` and iteration in order; `union(ranges)` builds
one. F19's `Steps<K>` is the same structure with a value on each stretch,
so the two should be one type or share their core. It needs S13 first.
Low priority on these counts: 2 problems here and 2 for F19.

### S19. The largest k, and descending order (4 of 10)

joi16fp2 adds up the `spare` largest gains:

```cp
sort(extra, by: s => -s)
print(reached - 1 + sum(extra[i] for i in 0..min(spare, extra.len)))
```

*Checked:* `sum(xs[..2])` gives `list index must be Int, got Range<Int>`:
a list has no slices, and the `min` is there because the list may be
shorter than `spare`. Wanted: `sum(largest(extra, spare))`. Descending
order is spelled by negating the key in four ports (`by: s => -s`, `by: b
=> -b`, `by: s => (-s.y, s.sweeps)`, and the heap of S11), which only
works for numbers and hides a mixed order in a tuple. Proposal:
`largest(xs, k)` and `smallest(xs, k)` (a selection, not a full sort);
`xs[..k]` and `xs.prefix(k)` on lists, clamped as `r.prefix(k)` on ranges
already is; and `by: desc(key)` for a descending key, usable inside a
tuple: `by: s => (desc(s.y), s.sweeps)`. In the corpus 22 problems sort
with `greater<>` or reverse iterators.

## Smaller notes

- A table over a size is rejected. *Checked:* `[n]Int` is a type, and
  `[n] i => 2` gives `table dimension must be a domain or range`; `[0..n]
  i => 2` then fails against the declared `[n]T`. sac22cc5p4 (dropped)
  declared two domains only to write its generators.
- `X(top - 1)` and `K(-k1)` turn a number back into a position four times
  in etsr23r1p2 and dmopc20c7p1. Both go away with S13 and S11; they are
  B1 of batch 4 seen from a workaround.
- `count(a != b for (a, b) in pairs)`, `print(list)`, `xs + ys`, `x in
  a`, `(0..=n).rev` and `where` clauses all worked the first time.

## Already good

- Sorting by a tuple key, and sorting records as they are: `sorted(ends)`
  in ccc21s3 orders by position and then weight with no comparator.
- The optional step: `if let before = first - 1` and `if let j = i + 1`
  never needed a bound test, and the two edges of coci11c4p4's fence fell
  out of `[i.prev, i.next]` with no special case.
- Euclidean `/`: joi16fp2's original has a helper `p_fdiv` for a division
  that must not round a negative number up. The port divides.
- Buckets over a range: `[0..=n]List<Int>` in cco17p5 is one line and
  needs no size arithmetic.
- `compress` with `first_ge` and `last_le` (D9 asks for the same in the
  library) made dmopc20c4p3 and etsr23r1p2 possible at all.
- Rethinking paid more than syntax did. Six of the ten ports are a
  different algorithm from their original, and those six are 118 lines
  today against 288; no wanted item above is needed for that.

# Friction backlog, batch 8: games, counting, modular arithmetic, bitsets

A Stage-1 scout pass over eleven submissions: two game DPs, three counting
problems, two permutation and XOR constructions, one big-number modular
problem, two bitset knapsacks and one meet-in-the-middle on a tree. The ports are in
[`proto/ports/scout_batch_8/`](../proto/ports/scout_batch_8/): dpk, dpl,
cco16p4, coci21c2p3, sac22cc4jp4, coci25c3p2, noip14p6, dpy, ioi04p5,
yac1p4 and cco19p3. Items are numbered `H1`... so they do not collide with `F`
(batch 1), `G` (batch 3) or `B` (batches 2 and 4).

What was checked: all eleven ports agree with their originals on 200
random cases each (`proto/ports/scout_batch_8/check.py`) on top of commit
`f4f53ad`. Eight build with `proto/cpc`; dpy, ioi04p5 and yac1p4 import
`stdlib/` and run on `proto/cpi` with small inputs only. Only cco19p3 was
run at full size (two hand-made inputs).
Every "today" snippet is a line a port contains. Every quoted message was
produced by the prototype named with it. The "wanted" snippets do not
build. A count is the number of the eleven ports with the awkward line; the
wider corpus was not counted unless a number says so.

## Suggested order for a Stage-2 sprint

1. H1, H2 and H17: a bound that is silently ignored, a call that builds
   C++ that does not compile, and a loop that never ends in one prototype
   and ends in the other.
2. H3: 3 of 11 could not be built by `proto/cpc` at all.
3. H4 and H5: the bitset knapsack step, 2 of 11.
4. H6, H7, H8: permutations, swapping and indexed loops, 2 of 11 between them.
5. H9, H10, H11: long numbers, words in a list, two moduli; all noip14p6.
6. H12, H13: bits of an `Int`, 2 of 11.
7. H18, H19, H20, H21: lists and parent arrays, all cco19p3.
8. H14, H15, H16: one port each.

## H1. A bound that names an earlier input is ignored (bug; coci25c3p2)

*Checked with `proto/cpc`:*

```cp
let n: 1..=5000 = read()
let k: 1..=n = read()
print(n, k)
```

`cp.py stress` generates `1` then `2`, and the built program reads it and
prints `1 2` with no trap. So `1..=n` is neither generated nor checked
for a single number. The port's first run failed with `domain value out
of range` three lines later. Today the port writes

```cp
let k: 1..=5000 = read() where k <= n
```

and 81 of 200 generated cases are thrown away. Proposal: a scalar bound
that names an earlier `let` is used by the generator and checked by local
runs, the same as a constant bound. 1 of 11 here; `1 <= k <= n` is on the
first line of most statements.

## H2. `f<P>(x)` is read as two comparisons (bug; noip14p6)

The first noip14p6 had a function over any modulus:

```cp
fn value<M>(s: Str) -> Mod<M>: ...
let ap = [value<P>(s) for s in a]
```

*Checked with `proto/cpc`:* this builds without a message and emits
`((value < P) && (P > (s)))`, which `g++` rejects. The spelling that
`decisions.md` leans to, `value::<P>(s)`, gives `expected an expression
(at ':')`. With the modulus only in an argument, `is_root(ap, x)` over
`coef: List<Mod<M>>` gives `cannot infer M for is_root from its
arguments`. Proposal: parse `f::<T>(...)`; reject `f<P>(x)` with a hint
when `f` is a generic function; infer a parameter that appears inside
`List<Mod<M>>`.

## H3. Ports still cannot use the standard library (3 of 11; batch 1 F1)

dpy needs `Comb`, ioi04p5 and yac1p4 need `Bitset`. *Checked:* `use
std.mod.{Mod, Comb}` in `proto/cpc` gives `expected end of line (at
'std')`. The three run on `proto/cpi`, which has its own gaps, and each
cost a rewrite:

- `let word: [domain I]Char = read()` gives `` `Fn` is not an index type ``;
  yac1p4 reads a `Str` and makes a list of numbers from it.
- `let walls: [Wall]Cell = read()` with `type Cell = (r: 1..=h, c: 1..=w)`
  gives `read() cannot read a Tuple<Int, Int>`; dpy writes the record type
  out in the declaration and again in the function that takes it:

  ```cp
  fn paths(a: (r: Int, c: Int), b: (r: Int, c: Int)) -> Z = ...
  ```
- `sorted(walls) ++ [(r: h, c: w)]` gives `expected an expression (at
  '+')`; dpy sorts into a `var` and pushes.
- `x - sum(... for j in 0..i)` with `i == 0` gives `` `-` needs two
  Mod<1000000007> values, got 0 ``: the sum of nothing is an `Int`. dpy
  subtracts in a loop.

coci21c2p3 builds with `proto/cpc` and for that reason fills its own
adjacency lists and walks them with a stack, 9 of its 23 lines, where
`std.graph` and `std.tree` have both. cco19p3 does the same for a tree
given by parents (H21). Proposal, unchanged from F1: one
prototype that both type-checks a solution and imports `stdlib/`. Until
then a bitset or combinatorics port is only ever checked on toy inputs.
For `sum`: the zero comes from the element type, so an empty sum of `Z`
is `Z(0)`.

## H4. A bitset shifts one way only (yac1p4)

The DP adds a letter's worth to some totals and subtracts it from others,
so the original shifts a `bitset` left and right around an offset.
`Bitset.shifted(k)` only moves positions up, and a `Bitset<D>` over
`-B..=B` cannot be shifted down. The port stores every total with the
worth of all letters seen so far added to it, so that all three moves
shift up:

```cp
next[open].union(reach[open].shifted(w))                               // letter left out
if let more = open + 1: next[open].union(reach[more].shifted(2 * w))   // added
if let less = open - 1: next[open].union(reach[less])                  // subtracted
...
for t in totals.each(): print(t.to_idx - whole)
```

That is a trick the reader has to verify. Wanted, over `domain Total =
-whole..=whole`:

```cp
next[open] = reach[open] | reach[more] << w | reach[less] >> w
for t in totals: print(t)
```

Proposal: `shifted` takes a negative count (or `<<` and `>>` both exist),
positions that leave `D` are dropped on either side, and iterating a
`Bitset<D>` yields `D` values, so a signed domain prints signed totals
with no offset. 1 of 11; signed-sum knapsacks are a standard bitset use.

## H5. The knapsack step is two method calls (2 of 11: ioi04p5, yac1p4)

```cp
for len in fields: reach.union(reach.shifted(len))
```

against `dp |= dp << arr[i]` in the original. `union`, `intersect`,
`subtract` and `symmetric` are statements that change the receiver, so
nothing composes into an expression, and a fresh empty set is spelled
`Bitset<Lifted>.new()`. yac1p4 also builds its rows by hand, twice:

```cp
var next = [Open] _ => Bitset<Lifted>.new()
```

Wanted: `reach |= reach << len`, and `var next: [Open]Bitset<Lifted> = []`.
Proposal: `impl BitOr, BitAnd, BitXor, Shl, Shr for Bitset<D>` in
`stdlib/bitset.cp`, so `x |= y` is the generic `x f= y` of the ergonomics
audit, and an empty-collection literal that fills an array with separate
empty values (batch 2 lists the same gap for `Set`). The operator traits
for bits wait on the `U64` decision.

## H6. A permutation has no cycles, and is not indexed by its values (sac22cc4jp4)

*Checked with `proto/cpc`:* with `let p: Perm<V> = read()`, `p[v]` gives
`index of type V into array over Pos<V>`. So the port reads `[V]V` with a
generator of its own, and walks the cycles by hand:

```cp
var seen: [V]Bool = false
for start in V:
    var cycle: List<V> = []
    var v = start
    while not seen[v]:
        seen[v] = true
        cycle.push(v)
        v = p[v]
    ...
```

Wanted:

```cp
let p: Perm<V> = read()
for cycle in p.cycles():
    for t, j in cycle: q[cycle[(t + k - 1) % cycle.len]] = j
```

Proposal: build `Perm<V>` as plan 19 §4.5 describes it (a sealed `[V]V`
with `inv`, `then` and `cycles`), and add `p.pow(k)` on top of `cycles`
for any `k`, negative included. 1 of 11. Among unported submissions
etsr23r4p1 also splits a permutation into cycles by hand.

## H7. Swapping two slots that may be the same slot (sac22cc4jp4; batch 1 F17, batch 2 B6)

The shuffle in the port's generator. *Checked with `proto/cpc`:*
`swap(mut a[v], mut a[w])` is refused, correctly, with `` `a[v]` and
`a[V(rand(1, n))]` may be the same storage, and both are lent `mut` to
this call ``, and `a.swap(v, w)` gives `[V]V has no method `swap``. Today:

```cp
let w = V(rand(1, n))
if v != w: swap(mut a[v], mut a[w])
```

Proposal: `a.swap(i, j)` on every `IndexedMut` (plan 19 already uses it
inside `Perm`), which is a no-op when `i == j`, and a prelude
`shuffle(mut a)`. With H6 the generator disappears, because `Perm<V>`
generates itself.

## H8. A loop over values and their places (2 of 11: sac22cc4jp4, dpy)

*Checked with `proto/cpc`:* `for t, j in cycle` gives `cannot destructure
V into 2 names`. Today:

```cp
for t in 0..cycle.len: q[cycle[(t + k - 1) % cycle.len]] = cycle[t]
for i in 0..stops.len:
    clean[i] = paths((r: 1, c: 1), stops[i])
```

Proposal: `for i, x in xs` for a `List` and for an array (where `i` has
the array's index type), as the one spelling; no `enumerate`.

## H9. A number too long for `Int`, wanted modulo a prime (noip14p6; batch 3 G9, G14)

Each coefficient has up to 10^4 digits. Today:

```cp
var v = 0
for c in s:
    if c != '-': v = (v * 10 + (c - '0')) % primes[t]
a[t, i] = if s[S.first] == '-': -v % primes[t] else: v
```

Wanted: `let a: [0..=n]Z = read()`. Proposal: reading into `Mod<P>`
accepts a decimal of any length, with a sign, and reduces as it reads
(G14 asked for the short case). For a modulus known at run time,
`ModR.parse(s)`. Smaller points on the way, all checked with `proto/cpc`:
`c.digit` does not exist and `c - '0'` is what works (G3); `s.first` gives
`[S]Char has no `.first``, so it is `s[S.first]`; in the generator
`Char('0'.ord + d)` gives `cannot call type Char` and `"0123456789"[d]`
gives `cannot index Str`, while `'0' + d` works.

## H10. A list of words (noip14p6)

*Checked with `proto/cpc`:* `let a: [0..=n]Str = read()` is accepted, but
`for c in s` gives `cannot iterate Str` and `s[i]` gives `cannot index
Str`; `[0..=n][domain]Char` gives `expected NAME (at ']')`. A word can
only be used as `[domain S]Char`, one binder per `read()`, so the port
reads inside its loop and reduces each word before the next one arrives:

```cp
for i in 0..=n:
    let s: [domain S]Char = read(gen: random_number)
```

Proposal: a `Str` is iterable and indexable by its own positions wherever
it is stored (plan 19 §4.4 makes it a flat array of `Char`), so
`[n]Str` works like `[n]List<Char>`.

## H11. The same code under two moduli (noip14p6)

The original instantiates `prune<1e9 + 7>()` and `prune<998244353>()`.
With H2 open the port cannot write one function over `Mod<M>`, so it
drops `Mod` and carries a prime index through plain `Int` arithmetic:

```cp
let primes = [1000000007, 998244353]
fn is_root(x: Int, t: Int) -> Bool:
    var y = 0
    for i in (0..=n).rev: y = (y * x + a[t, i]) % primes[t]
    y == 0
```

It reads acceptably, but every `% primes[t]` is one that can be
forgotten. Wanted, once H2 and H9 are fixed:

```cp
fn roots<M>() -> [1..=m]Bool:
    let a: List<Mod<M>> = ...
    [1..=m] x => fold(a.rev, (y, c) => y * x + c) == 0
```

Proposal: generic functions over a constant, called as `roots::<P>()`.
Double-modulus checks and double hashing are the uses.

## H12. Counting the bits of an `Int` (coci21c2p3)

*Checked with `proto/cpc`:* `popcount(flip)` gives `unknown name
popcount` (`proto/cpi` has it; `stdlib/bitset.cp` calls it). Today:

```cp
fn even(bits: Int) -> Bool = count((bits >> k) % 2 == 1 for k in 0..n) % 2 == 0
```

Proposal: `popcount`, `ctz` and `parity` in the prelude of both
prototypes, for `Int` now and for `U64` when that decision lands.
This batch's search found 12 unported originals that use `popcount` or
a `bitset`.

## H13. A set of edges that must also be a number (coci21c2p3)

The label of a vertex is the set of edges on its path from the root, and
the answer prints it as a number. The port keeps plain `Int`s:

```cp
adj[edges[e].u].push((edges[e].v, 1 << e.idx))
label[v] = label[u]! ^ bit
```

`Subset<E>` is the type that says what this is, but a port cannot print
it as a number or `^` two of them. This was not probed beyond batch 4's
B2 and B6, which found `a - b` on two subsets to be a number. Proposal:
`Subset<D>` has `^` (symmetric difference) next to the set algebra B6
asks for, `s.bits` gives the `Int`, and `Subset<D>.single(x)` replaces
`1 << x.idx`.

## H14. A step that cannot fail still needs `!` (dpl)

```cp
if l == r: return a[l]
max(a[l] - lead((l + 1)!, r), a[r] - lead(l, (r - 1)!))
```

`l + 1` is optional because it may leave `I`. Here `l < r`, so it cannot,
but that is true only because of the `return` above and the unstated
`l <= r`. Batch 4's dpn avoided the same line by indexing with cuts,
which does not fit a game where the ends are the items taken. Proposal:
flow facts about positions (`l != r` together with a declared `l <= r`)
discharge the optional, the way plan 19 §5.4 uses `i != j` for disjoint
lends. A parameter pair `l..=r: Range<I>` that is never empty would
carry the `l <= r`. 1 of 11; every interval DP on closed ends has it.

## H15. Rows and columns of a grid as sequences (cco16p4)

The key of a grid is the parity of every row and column. The first
version passed a generator to a helper, `fn odd(cells: Iter<Char>)`.
*Checked with `proto/cpc`:* `unknown type Iter`. Today the two lines are
written out:

```cp
let rows = [count(grid[r, c] == 'R' for c in Col) % 2 for r in Row]
let cols = [count(grid[r, c] == 'R' for r in Row) % 2 for c in Col]
```

Wanted: `[count(x == 'R' for x in line) % 2 for line in grid.rows()]`
and the same with `grid.cols()`. Proposal: `rows()` and `cols()` on a
grid, each lending a line (`g.row(r)` exists; a column view does not),
and a nameable type for "any sequence of `T`" in a parameter.

## H16. Reading many grids (cco16p4)

The port reads one grid per turn of a loop and keeps a running counter,
because the pairs are counted as the grids arrive. With the grids in an
array the count is one expression:

```cp
let grids: [g]Grid = read()
print(sum(c * (c - 1) / 2 for c in counts(key(x) for x in grids).values))
```

This form was not probed, so it may already build. It is listed because
`counts(...)` has no documented way to iterate its counts; the audit only
shows `c[k]` and `c.distinct`.

## H17. `for i in 0..xs.len` while `xs` grows: the two prototypes disagree (bug; cco19p3)

The first `sums` in cco19p3 doubled a list in place:

```cp
var out = [0]
for x in xs:
    for i in 0..out.len: out.push(out[i] + x)
```

*Checked:* `proto/cpc` emits `for (ll i = 0; i < (ll)out.size(); ++i)`,
so the end moves with every push and the loop never ends; the stress run
timed out. `proto/cpi` reads `out.len` once and stops. With `if out.len <
50:` in front of the push and `xs = [1, 2]`, the loop leaves 50 elements
under one prototype and 4 under the other. Proposal: decide it in `decisions.md` (a range is a value,
so its ends are read once) and make `proto/cpc` follow. A loop whose
bound silently chases the list it fills is the kind of 1% case the borrow
rules exist to stop: `out` is being read by the range and changed by the
body.

## H18. Appending one list to another (cco19p3, dpy)

*Checked with `proto/cpc`:* `out = out ++ [s + x for s in out]` gives
`expected an expression (at '+')`, `out.extend(...)` gives `List<Int> has
no method `extend``, and `out += [...]` gives `` `+=` between List<Int>
and List<Int> ``. Today:

```cp
for more in [s + x for s in out]: out.push(more)
```

Proposal: one spelling in both prototypes, `xs ++ ys` for the value and
`xs ++= ys` through the generic `x f= y`. dpy wanted the same line as
`sorted(walls) ++ [(r: h, c: w)]` (H3). 2 of 11.

## H19. Binary search in a sorted list (cco19p3)

The original calls `upper_bound` on the sorted half-sums. *Checked with
`proto/cpc`:* `ls.partition_point(l => l <= x)` gives `List<Int> has no
method `partition_point``, and so does `last_where`; both exist on a
domain only. The port turned the lookup into a sweep with a moving index:

```cp
var i = 0                                  // ls[i]: the largest that fits beside r
for r in rs.rev:
    if 2 * r > whole: continue
    while i + 1 < ls.len and 2 * (ls[i + 1] + r) <= whole: i += 1
    best = max(best, ls[i] + r)
```

It is correct and linear, but it took thought the original did not need.
Wanted:

```cp
max(r + ls.last_where(l => 2 * (l + r) <= whole)! for r in rs if 2 * r <= whole)
```

Proposal: `first_where`, `last_where` and `partition_point` on anything
indexed, a `List` included (batch 1 F10 asked the same for a plain
range), each giving the element or position as an optional. 1 of 11;
`lower_bound` or `upper_bound` on a vector is in most sorted-list
originals.

## H20. The two halves of a list (cco19p3)

*Checked with `proto/cpc`:* `parts[..cut]` and `parts[0..cut]` give `list
index must be Int, got Range<Int>`, and `parts.split_at(cut)` gives
`List<Int> has no method `split_at``. Today:

```cp
let ls = sorted(sums([parts[i] for i in 0..cut]))
let rs = sorted(sums([parts[i] for i in cut..parts.len]))
```

Proposal: a `List` is sliced by a range like an array is (`parts[..cut]`,
`parts[cut..]`, lent, not copied), per plan 19's indexed slices. With
that, a prelude `subset_sums(xs)` is three lines of userland and
meet-in-the-middle is `subset_sums(parts[..cut])` twice.

## H21. A tree given as a list of parents (cco19p3; batch 1 F1)

The input is `p[2..=n]`. Getting from there to "sizes of subtrees" took
ten lines: child lists, a top-down order built with a `while` and a
counter, and a scan of that order backwards.

```cp
domain Kid = 2..=n
let parent: [Kid]V = read(gen: random_parents)
var kids: [V]List<V> = []
for v in Kid: kids[parent[v]].push(V(v.idx + 2))
var order = [V.first]
var at = 0
while at < order.len:
    for c in kids[order[at]]: order.push(c)
    at += 1
```

Two smaller things are in there. A `Kid` is a `V` by construction, but
*checked with `proto/cpc`:* passing `v` gives `expected V, got Kid`,
`V(v)` gives `V(x) needs an Int, got Kid`, and `Int(v)` gives `cannot
convert Kid to Int`; `V(v.idx + 2)` works and has a constant in it that
must match the domain's first value. And without the named domain, the
generator's `[2..=n]V` and the declaration's `[2..=n]V` are different
types: `expected [range]V, got [range]V`.

Wanted:

```cp
let tree = Rooted.from_parents(read())      // p[2..=n], each a V
let below = tree.fold_up(a, (+))            // sum of a over each subtree
```

Proposal: `Rooted.from_parents` and a bottom-up fold in `std.tree` (the
struct already keeps `tin`, `size` and `depth`), which needs H3 first. A
value of a domain that lies inside another converts with `V(v)`; the
ergonomics audit already lists conversions between related domains. Two
anonymous ranges with the same bounds are the same type, or the message
names the fix (`domain Kid = 2..=n`).

## Seen again from earlier lists

- **F1** (no standard library in `proto/cpc`): H3, H21.
- **F10** (search over something that is not a domain): H19.
- **F17, batch 2 B6** (two slots of one array): H7.
- **G3** (the value of a digit), **G9** (numbers past `Int`), **G14**
  (reading into `Mod`): H9.
- **Batch 4 B2, B6** (set algebra on `Subset`): H13.
- **Batch 2, "filling an array with a set"**: H5.

## Already good

- `wins[s] = any(s >= x and not wins[s - x] for x in a)` is the whole
  recurrence of dpk, and `@memo fn lead(l: I, r: I)` the whole table of dpl.
- `Counter` takes a pair of lists as a key (cco16p4), so "seen this shape
  before" needed no hash and no encoding into a mask.
- Euclidean `%`: `-v % p` in noip14p6 and `(t + k - 1) % cycle.len` with
  `k` up to 1e18 in sac22cc4jp4 are right with no fix-up.
- `ways[c] * (i - 1)` mixes a `Mod` and an `Int` without a cast, and
  `Comb<P>.new(h + w)` replaced about 25 lines of dpy's original.
- `Bitset<Taken>` over `0..=q` drops shifted-out positions by its type,
  so ioi04p5 needs no size constant.
- `read(gen: ...)` and `where` kept every generator next to the input it
  describes.
- `for v in order.rev` and a `[V]Int` table made the two tree passes of
  cco19p3 plain loops with no recursion and no stack depth to worry about.

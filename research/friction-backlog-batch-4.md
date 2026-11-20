# Friction backlog, batch 4: dynamic programming

A Stage-1 scout pass over ten DP submissions: subset, digit, interval,
knapsack, permutation, connected-component and tree DP. The ports are in
[`proto/ports/scout_batch_4/`](../proto/ports/scout_batch_4/): dpu, dpo,
dps, dpn, dpx, dpm, dpt, joi16op3, ioi06p4 and noi02p3. This file follows
the format of [`friction-backlog.md`](friction-backlog.md) (batch 1), and
its items are numbered `B1`... so they do not collide with batch 1's `F`
numbers. An item that batch 1 already has is named with its `F` number.

What was checked: all ten ports build with `proto/cpc` at commit `c217120`
and agree with their originals on 200 random cases each
(`proto/ports/scout_batch_4/stress.sh`) and on one largest-size input each.
So every "today" snippet below is the line a port really contains, or one
it contained before it was rewritten. Every error message quoted was
produced by a small probe with `proto/cp.py build` at that commit. The
"wanted" snippets do not build. A count is the number of the ten ports
that contain the awkward line; other ports and batches are named
separately, and the wider corpus was not counted.

## Suggested order for a Stage-2 sprint

1. B2 and B3, because they are a wrong answer without a message and a
   build that emits C++ that does not compile.
2. B1: 4 of 10. It is advice and one error message, no new feature.
3. B4: 5 of 10 write a `next` table and assign it back.
4. B6, B7, B8: one library piece each; 2, 1 and 2 of 10.
5. B9, B10, B11, B12: small typing and parser changes; 2, 2, 1 and 1 of 10.
6. B13, B14: 1 of 10 each.
7. B5 waits for decision 1.5.

## B1. Amounts are ranges, names are domains (4 of 10)

dps, dpx, joi16op3 and noi02p3. Every earlier port declares a `domain` for
every table dimension, so these four were first written that way. For a
dimension that counts something (groups so far, cost so far, a remainder),
the recurrence does arithmetic on the index, and a domain value has to be
unwrapped for it and wrapped again. *Checked:* `i * 2` on a domain value
gives `` `*` between I and Int[2..=2]; domain values support `i + k` and
`i - k` (a ?I) and `i - j` (an Int) ``, and `w[0] = 1` gives ``Int index
`0` into array over I; convert with I(x) or test `x in I` ``. The first
joi16op3 had eight `.idx` and five constructor calls:

```cp
domain Groups = 0..=n
domain Cost = 0..=limit
domain Ends = 0..=2
var ways: [Groups, Cost, Ends]Z = 0
ways[Groups(0), Cost(0), Ends(0)] = 1
...
let sides = 2 * g.idx - e.idx
if let paid = c + (h - level) * sides and w != 0:
    if let more = g + 1:
        next[more, paid, e] += w * (g.idx + 1 - e.idx)
        if let ends = e + 1: next[more, paid, ends] += w * (2 - e.idx)
```

and the first dps wrote `next[Rem((r.idx + x.idx) % d)] += below[r]`.
The prototype already takes a plain range as a dimension, with `Int`
indices that are checked when the program runs. The ports now use that,
and it builds today:

```cp
type Ways = [0..=n, 0..=limit, 0..=2]Z
var ways: Ways = 0
ways[0, 0, 0] = 1
...
let sides = 2 * g - e
let paid = c + (h - level) * sides
if w != 0 and paid <= limit:
    next[g + 1, paid, e] += w * (g + 1 - e)
    if e < 2: next[g + 1, paid, e + 1] += w * (2 - e)
```

Nothing new is needed for this, but nothing points a writer to it either.
Proposal, in three parts. (1) Say the rule in the docs and in
`research/ports-report.md`: a `domain` names things (cities, positions,
rabbits), a range counts things, and a table dimension can be either.
(2) Add the hint to the two errors above: "if I is an amount and not a name,
index by the range `0..=n`". (3) A lint for a domain whose values are only
built from numbers and only read through `.idx`. Earlier ports with the same
shape: coci20c6p3 (`Pairs`, and `choose[Size(size), Size(take)]`),
ceoi18p1 (`Spare`) and noip18p2 (`Amount(c)`). This also covers the open
item "domain values used as numbers" in `friction.md` for the
cases that are amounts.

Two things are still awkward in the range form and are items B9 and B10.

## B2. `a - b` on two subsets is a number (bug; dpu)

*Checked.* With `type S = Subset<I>`, `S.full - S.of(I.first)` builds, and
its type is `Int[-32..=32]`: the two masks are subtracted as if they were
positions. Anyone who writes set difference this way gets a number, and
`best[s - g]` then fails with ``Int index `s - g` into array over
Subset<I>; convert with Subset<I>(x)``, which does not mention sets and
whose advice gives the right set only when `g` lies inside `s`. Proposal: until B6 lands, `-` between two `Subset<D>` values
is a compile error that names `.without`. With B6 it is set difference.

## B3. Assigning a row builds C++ that does not compile (bug; noi02p3)

*Checked.* `cost[p] = merged` on a 2-D table gives `` [I, I]Int is indexed
as g[i, i]; for a whole row use .row(...) ``. Following that advice,
`a.row(I.first) = row` passes `cp.py build`, and then `g++` rejects the
output with `no match for 'operator=' (operand types are 'CpView<long long
int>' and 'std::vector<long long int>')`. Today noi02p3 copies back cell by
cell:

```cp
for w in Who:
    for x in 0..=n: cost[p, w, x] = merged[w, x]
```

Wanted: `cost.row(p) = merged`, for a table with any number of
dimensions left over. Proposal: make assignment to `.row(...)` copy
element by element, and let `.row` fix a prefix of the dimensions.
`friction.md` already has "arrays cannot be refilled with broadcast
assignment" open; this is the same hole from the other side. Simplifies 1
problem here.

## B4. A fresh table for every layer (5 of 10, 2 earlier ports)

dps, dpm, dpt and joi16op3 declare `next`, fill it and assign it back;
noi02p3 does the same with `merged`. Earlier: coci20c6p3 (`var next` ...
`ways = next`) and ceoi18p1 (`let before = best`). The declaration repeats
the table's type and fill value, and the assignment at the end of the loop
body is easy to forget or to indent wrongly:

```cp
for p in Pos:
    var next: Row = 0
    for r in 0..d:
        for x in Digit: next[(r + x.idx) % d] += below[r]
    ...
    below = next
```

Where each new cell is a function of the old row, a table generator already
says it in one line, and B8 would let dpm and dpt use that. The other three
push from each old cell into several new ones, and that has no short form.
Wanted, as a library function over `inline` callbacks (the same mechanism
`lends` projections use, so no new syntax):

```cp
below = below.next_layer(0, next =>
    for r in 0..d:
        for x in Digit: next[(r + x.idx) % d] += below[r])
```

Proposal: `a.next_layer(fill, body)` in `stdlib/`, which makes a table of
`a`'s shape filled with `fill`, lends it mutably to `body`, and returns it.
The prototype cannot express it yet. *Checked:* a function-typed parameter
(`fn apply(f: fn(Int) -> Int)`) gives `expected an expression (at 'fn')`,
and a lambda body is one expression. If those stay out of the prototype,
the fallback is `a.blank(fill)`, which only removes the repeated type. Simplifies 3 problems here with the push
form and 2 more through B8.

## B5. Floats (1 problem dropped; batch 1 F15; decision 1.5 is open)

dpj (Sushi, `4131649.cpp`) is an expected-value DP over how many dishes
hold one, two and three pieces. *Checked:* `F64` and `Float` are unknown
types. The port was written and then dropped, because it cannot be built or
compared. This is what it would be:

```cp
@memo
fn steps(one: 0..=n, two: 0..=n, three: 0..=n) -> F64:
    let left = one + two + three
    if left == 0: return 0.0
    var total = F64(n)
    if one > 0: total += one * steps(one - 1, two, three)
    if two > 0: total += two * steps(one + 1, two - 1, three)
    if three > 0: total += three * steps(one, two + 1, three - 1)
    total / left
```

Besides `F64` itself it needs two smaller things: a flat `@memo` table over
range parameters (B11), and a default print precision that passes the usual
`1e-9` checkers. Probability and expected-value DP is a whole family, so
this blocks more than one problem, but only one was tried here.

## B6. Subsets of a subset, and set algebra (1 of 10)

dpu. `Subset<D>` already covers "for each member", `in`, `.with(x)`,
`.without(x)`, `.len` and `.is_empty`, which is all dpo needs (see
"Already good"). Partition DP also needs every subset of a subset and the
difference of two subsets. *Checked:* `s.subsets` gives `Subset<I> values
have no .subsets`, and `.without` takes one member only. So today dpu goes
back to raw masks:

```cp
for s in Group:
    var g = s.idx
    while g > 0:
        best[s] max= score[Group(g)] + best[Group(s.idx - g)]
        g = (g - 1) & s.idx
```

Wanted:

```cp
for s in Group:
    best[s] = max(score[g] + best[s - g] for g in s.subsets if not g.is_empty) ?? 0
```

Proposal: on `Subset<D>`, `s.subsets` (every subset, by the `(g - 1) & s`
walk), `s - g`, `s | g`, `s & g` and `g <= s`. All of it is userland over
the mask ordinal. Other submissions looked at for this batch and not
ported: noip00p4 (`5067920.cpp`) does sums over subsets and btoi17p1
(`4456179.cpp`) intersects neighbour masks, and both need the same
operators. Simplifies 1 problem here.

## B7. Steps that wrap (1 of 10; batch 1 F11)

ioi06p4 walks a circle of cities and defines both neighbours by hand:

```cp
fn cw(c: City) -> City: (c + 1) ?? City.first
fn ccw(c: City) -> City: (c - 1) ?? City.last
```

*Checked:* `c.cyc(1)` gives `I has no method cyc`, though `plan/12`
specifies it. Wanted: `c.cyc(1)` and `c.cyc(-1)` in place, and no helper
functions. Proposal: implement `v.cyc(k)` as specified. Batch 1 has it as
F11 for ccc22s4, and btoi17p3 has the same shape on a list. Simplifies 1
problem here, 3 with those two.

## B8. `prefix` of a modular table (2 of 10; extends batch 1 F2)

dpm and dpt. Both replace each cell by the sum of a stretch of the row
before, which is `prefix` followed by a range sum. *Checked:*
`prefix(ways)` gives `prefix takes a 1-D or 2-D array of Int, got
[I]Mod<1000000007>`. So dpm keeps a sliding window by hand:

```cp
for most in a:
    var next: [Given]Z = 0
    var window = Z(0)                   // ways[g - most ..= g]
    for g in Given:
        window += ways[g]
        if let gone = g - (most + 1): window -= ways[gone]
        next[g] = window
    ways = next
```

and dpt has two loops, one running up and one running down, for its two
signs. Wanted:

```cp
for most in a:
    let before = prefix(ways)
    ways = [Given] g => before.sum((..=g).suffix(most + 1))
```

and for dpt:

```cp
let before = prefix(ways)
ways = [Rank] r => if r > placed: 0 elif signs[g] == '<': before.sum(..r) else: before.sum(r..)
```

Proposal: `prefix` over any element type that has `+` and `-`. Counting DP
is modular almost always, so `Int` only leaves out most of the DP uses.
Batch 1's F2 asks for the same over `Bool` and for a running `scan`; one
generic `prefix` covers all three. Simplifies 2 problems here: 7 lines to 2
in dpm, 11 to 2 in dpt.

## B9. Two tables over the same range are different types (2 of 10)

dps and joi16op3, once they use ranges (B1). *Checked:* with `var below:
[0..d]Z = 0` and `var next: [0..d]Z = 0`, the line `below = next` gives
`expected [range]Mod<1000000007>, got [range]Mod<1000000007>`. Each
written range is its own type, and the message prints both the same. Today
the fix is an alias that the reader has to look up:

```cp
type Row = [0..d]Z
var below: Row = 0
```

Proposal: two array types over ranges with the same written bounds are the
same type. Failing that, print the bounds in the message and suggest the
alias. Simplifies 2 problems here, and any layered DP written with ranges.

## B10. A number and a modular number in two branches (2 of 10)

joi16op3 and dps. A literal becomes a `Mod` where the type is known: `var
ways: [I]Z = 0`, `ways[i] = 1`, `x += 1` and `w != 0` all build, and the
ports use them. It stops at `if`. *Checked:* `print(if n == 1: 1 else:
sum(ways[1, c, 2] for c in 0..=limit))` gives `expected Int[1..=1], got
Mod<1000000007>`, so joi16op3 writes `Z(1)` there, and `var window = Z(0)`
in dpm and `var run = Z(0)` in dpt need the constructor for the same
reason: nothing else says the type. Proposal: the branches of an `if`
expression unify a literal with a `Mod` the way assignment does. Simplifies
2 problems here.

## B11. `Bool` and plain enums as dimensions and `@memo` keys (2 of 10)

ioi06p4 and noi02p3. ioi06p4 memoises on `(l: City, r: City, at_r:
Bool)`. *Checked:* it builds, but the generated table is an `unordered_map`: a
`Bool` parameter makes `@memo` fall back to hashing without saying so,
where a flat `[City, City, 2]` table is meant. A range parameter (`fn f(a:
0..=n)`) hashes in the same way. noi02p3 wants "a small head or the big head" as a dimension.
*Checked:* `enum Who: Small, Big` followed by `var t: [Who]Int = 0` gives
`array dimension must be a domain, a range, or a struct with impl Index,
got Who`. So it invents a numbered domain and names one value:

```cp
domain Who = 0..2                       // 0: a small head, 1: the big head
let big = Who(1)
cost[v, Who(0), 0] = 0
```

Wanted:

```cp
enum Who: Small, Big
var cost: [Fruit, Who, 0..=n]?Int = none
cost[v, Small, 0] = 0
```

Proposal: every finite type is an index. `Bool`, an enum whose variants
carry nothing, and a range are dimensions and flat `@memo` keys, through
the same `impl Index` a domain has. Simplifies 2 problems here. Flags like
"tight", "started" and "at the right end" are in most digit and interval
DPs.

## B12. `let` only at the front of a condition (1 of 10)

joi16op3, in its first form. *Checked:* `if w != 0 and let paid = c + ...:`
gives `expected an expression (at 'let')`. The cheap test has to come
second, after the step it was meant to skip:

```cp
if let paid = c + (h - level) * sides and w != 0:
```

Proposal: `let` anywhere in an `and` chain, bound for what follows it.
Simplifies 1 problem here.

## B13. The input's shape, and its alphabet (1 of 10; batch 1 F21)

dpt reads `n` and then `n - 1` signs, each `<` or `>`. *Checked:* `[n -
1]Char` gives `expected a type` (batch 1's F21), and `domain Sign = {'<',
'>'}` gives `expected '}' (at ',')`. Leaving both out, the stress
generator made `n = 2` with the word `ae`. So the port declares a domain
for the count and writes its own generator:

```cp
domain Gap = 1..n
fn random_signs() -> [Gap]Char: [Gap] _ => if rand(0, 1) == 0: '<' else: '>'
let signs: [Gap]Char = read(gen: random_signs)
```

Wanted:

```cp
enum Sign: Less = '<', More = '>'
let signs: [n - 1]Sign = read()
```

Proposal: F21's "any expression as an array count", plus enums whose
variants are single characters, which `read()` can parse and generate.
Grids of `#` and `.` and move strings like `LRUD` are the same thing.
Simplifies 1 problem here.

## B14. A rooted tree forgets its edge data (1 of 10; batch 1 F16)

noi02p3. `Rooted(adj, root)` takes `[V]List<V>`, so the branch costs are
stored a second time, by finding the lower end of each edge:

```cp
var adj: [Fruit]List<Fruit> = []
for (a, b, _) in branches:
    adj[a].push(b)
    adj[b].push(a)
let tree = Rooted(adj, Fruit.first)
var up: [Fruit]Int = 0                  // cost of the branch to the parent
for (a, b, cost) in branches:
    up[if tree.parent(a) == b: a else: b] = cost
```

btoi17p3 has the same `lower(t)` helper. Wanted:

```cp
let tree = Rooted(branches, Fruit.first)
... tree.up(c).cost ...
```

Proposal: `Rooted` takes the edge list as read, any record with two
vertex fields, and `tree.up(v)` gives the edge to the parent. This removes
the adjacency loop as well, which the audit lists under "graph
boilerplate". Simplifies 1 problem here and 1 earlier port, and it is the
general form of batch 1's F16.

## Smaller notes

These cost a line or less each, and each came up once.

- Top-level early exit, open in `friction.md`. noi02p3 ends with
  `print(if heads - 1 + k > n: -1 else: ...)` and joi16op3 with `if n == 1:
  Z(1) else: ...`, where the original returns early. ioi06p4 wraps its
  search in `fn route() -> ?List<City>` to be able to return.
- `path.last` on a `List` gives `List<City> has no .last`; it is `.last()`
  there and `.last` on a domain.
- A range between two cuts gives positions, which is right, but there is no
  range of the cuts strictly between two cuts. dpn writes `for m in l..r if
  m.l != l` and uses `m.l`. Wanted: `for m in l.between(r)`.
- joi16op3 needs each height with the one before it and keeps `var level`
  for that. `adjacent(xs)` is an unknown name; batch 1's F7 covers it.

## Already good

Things these ports needed that built on the first try, for the record:

- `Subset<D>` as a table dimension, in subset order: `for t in Taken`, `for
  w in t`, `t.without(w)`, `t.len`. dpo is 14 lines and has no shift or
  mask in it.
- `sum(a[i, j] for i in g for j in g if i < j)` over the members of a
  subset.
- `@memo` over a pair of cuts (`I.bounds`), with `prefix(a).sum(l..r)`:
  dpn's whole recurrence is two lines.
- `max=` and `min=` on a `?Int` cell, and `max(best)` over a table of
  `?Int`, which skips the empty cells. dpx and noi02p3 have no "infinity"
  constant.
- `for x in Digit.first..k[p]` over a domain of digit characters, and
  `(0..=b.s).rev` for a knapsack that runs downwards.
- `sorted(blocks, by: b => b.s + b.w)` and reading a record per line.
- Rebuilding a route by calling the memoised function again (ioi06p4), so
  no table of choices is kept.
- `tree.preorder.rev` for "children before parents" (noi02p3).

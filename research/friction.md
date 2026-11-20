# Friction report: 30 corpus problems re-solved in `proto/cpc`

Thirty accepted C++ submissions from `corpus/` are now re-solved in CP Lang
under `proto/ports/` (11 from the earlier round, 19 new in this one). Every
port compiles, and `proto/run_all.sh` checks each against its original on
random inputs. Line counts and test status per port are in
[`ports-report.md`](ports-report.md); this file is what the ports taught us
about the language.

Each item names the port it came from and says where it stands: **fixed**
(the prototype changed and the port uses it), **prototyped** (works in
`proto/cpc`, but as a compiler builtin that should become library code), or
**open** (a decision or a piece of work for the Rust compiler).

## The six changes that matter most

### 1. A range that depends on the input is not enforced (open, noip14p3)

This is the most serious finding. A declared range may mention an earlier
input:

```cp
let n: 1..=10000 = read()
let k: 0..n = read()
let pipes: [k](at: X, low: 0..=m, high: 0..=m) = read()
```

The prototype widens `0..n` to its static worst case, `0..=9999`, for both
the input check and the test generator. So `k = 7` with `n = 2` is accepted
at run time, and the generator produces it. The precondition reads as if it
were checked, and it is not. The port has to say it twice:

```cp
fn fewer() -> Int: rand(0, n - 1)
let k: 0..n = read(gen: fewer) where k < n
```

The static interval is the right thing for overflow analysis. The run-time
check and the generator must use the actual bound. With that, most of the
`where` clauses and half of the custom generators in the ports disappear.

### 2. Moving inside a domain still drops to raw integers (open; noi14p4, noip14p3, noip18p2, coci20c6p3, btoi17p6, aac5p2)

Domains remove index arithmetic when the code visits elements. They do not
help yet when the code *moves* by a computed amount, and that is where every
remaining `.idx`, `.label` and `D(...)` conversion in the ports sits:

```cp
next[H(min(h.label + up, m))] min= t + 1            // noip14p3: move up, stop at the ceiling
if h.label > down: next[H(h.label - down)] min= t   // noip14p3: move down unless it leaves H
if a.idx >= c and payable[a - c]: ...               // noip18p2: guard written by hand
while b.idx > 0 and s[S(b.idx)] != s[i]: ...        // noi14p4: from a bound to the element after it
choose[i, Size(j)] = choose[Size(i.idx - 1), Size(j - 1)] + ...   // coci20c6p3
```

Three small operations would remove nearly all of them: a checked offset
that returns an optional (`h - down` is `?H`, so `if let low = h - down`), a
saturating offset (`h.up(by, clamp)`), and the inverse of `i.before` on a
bound (`b.after_elem -> ?S`). The KMP port (noi14p4) is the worst case: its
state is "a prefix length", which is `S.bounds`, and it converts between a
bound and a position six times. This belongs in the bounded-index
unification draft: an offset on an index type is one operation, whatever
the type is called.

A related case is a second domain that is really part of the first. In
btoi17p6 the input gives a parent for nodes 1..n-1 only, and in aac5p2 the
questions are about the n-1 gaps between neighbours. Both needed a second
domain (`Kid`, `Gap`) and conversions such as `I(g.label + 1)`. A sub-domain
whose values *are* values of the parent (`V.rest`, `I.pairs`) would remove
them.

### 3. The standard library is where the lines go, and each piece is small (prototyped)

The library-stripped number is the honest one, and it moved most where a
port could name a structure instead of writing it. Each of these is a
builtin in the prototype and should be ordinary library code in the real
compiler:

```cp
var gcds = SegTree<I, Int>(0, (a, b) => gcd(a, b))    // ccc21s5: 17-line sparse table
gcds.query(l, r)                                      // inclusive, takes domain values

var depth = AddMax<X>()                               // noi16p4: 25-line lazy tree
depth.add(lo, hi, 1); depth.max()

let tree = Rooted(adj, City.first)                    // btoi17p3: 30 lines of tin/tout/lifting
tree.lca(a, b); tree.parent(c); tree.pos(c); tree.preorder

let hash = StrHash(hay)                               // ccc20s3: 45 lines of modular hashing
hash.of(start, end)

type Used = Subset<Colour>                            // btoi18p6: bitmask DP with no shifts
if colour[v] in s: ways[s, v] = sum(ways[s.without(colour[v]), u] for u in adj[v])
```

A monoid given as two plain values (an identity and a lambda) was enough for
`SegTree`; no trait or struct was needed. `AddMax` is the one that will not
generalise that way: a lazy tree needs a monoid plus an action on it, and
that is the first place in the corpus where a trait earns its place.

Still hand-written in ports, and worth a library entry: weighted union-find
(noi02p1, which is why that port is longer than its stripped original),
binomials modulo a prime (coci20c6p3), the prefix function (noi14p4),
multi-source Dijkstra with starting costs (cco24p1), and 2-D vectors with
`+`, `-` and `cross` (cco16p2).

### 4. Strings are `[domain S]Char`, and that is enough (fixed; ccc20s3, noi14p4, coci20c6p3)

`decisions.md` leans towards `Str` being a domain-indexed array of
characters. The prototype could only read an opaque string, so this round
implemented the decision:

```cp
let needle: [domain N]Char = read()
let hay: [domain H]Char = read()
for c in word: letters[Letter(c)] += 1
```

Everything that works on arrays then worked on strings with no string-only
rule: iteration, indexing by position, `N.len`, bounds, hashing. Reading a
word inside a loop, each with its own fresh domain, also works
(coci20c6p3). This supports removing `Str` as a compiler feature.

### 5. Optionals replace sentinels, but only where the short form is known (fixed)

`[H]?Int` with `min=` removed three different "infinity" constants from
noip14p3 (`0x3f3f3f3f`, `1e9` and a `< 1e9` test). The pieces had gaps, now
closed: `min`/`max` over optionals skip `none`, `taps[h] = none` typechecks,
`?T == T` means "present and equal", and `List.pop()` returns `?T` like the
heap, so `while let u = stack.pop()` works for both.

One lesson is about discoverability rather than design. In noi16p4 the
first draft was `best = min(best ?? 2e9, x)`, a sentinel again, because it
was not obvious that `best min= x` already treats `none` as "no candidate
yet". The tour should show that line.

### 6. Interactive problems need one construct, not a protocol (prototyped, aac5p2)

The prototype had no way to ask a question. One form covers it:

```cp
let p: 1..=n * n = ask("?", g, g.label + 1)
```

`ask` prints the line, flushes, and reads a reply of the declared type, so
the reply is range-checked like any input and a forgotten flush cannot
happen. It also exposed a runtime bug: the input reader used `fread`, which
waits for a full buffer on a pipe, so any interactive program would have
hung on a real judge. Tests run against a small judge written in Python
(`proto/interact.py`, `proto/ports/aac5p2_judge.py`); the port and the
original must both be accepted.

## Smaller items

**Sorting has two spellings, and needs a decision (prototyped, cco16p2).**
`sort(xs, by: x => key)` covers every port except the angular sort, which
has no key, only a comparison: `sort(seen, less: (a, b) => cross(a.dir,
b.dir) > 0)`. Either keep `less:` for orders with no key, or require a key
type with `<`.

**Counting with a map (fixed, coci13c1p3).** `seen[x] += 1` on a missing key
trapped; a compound assignment on a missing numeric key now starts from
zero. Without it every counting loop is four lines.

**Binary search as an expression (prototyped, aac4p2).** A nine-line lo/hi
loop became `I.first_where(i => ...)`, which returns `?I`, so "no such
index" needs no special value. `partition_point` returns a bound and is the
other half.

**`gcd` and `lcm` (fixed).** An earlier port had written its own.

**Read, then sort, needs two names (open; yac1p1, dmopc21c2p1).** `let a_in:
[R]Int = read()` followed by `let a = sorted(a_in)`. `sorted(read())` with
the declared type flowing inward would do.

**An array cannot be refilled (open, coci20c6p3).** `var ways: [Pairs]Z =
Z(0)` is allowed at the declaration, but `ways = Z(0)` later is a type
error. Either allow the same broadcast on assignment or add `ways.fill(x)`.

**A table needs a domain (open, noi16p4).** `[n]Span` can be read, but a
generator cannot build one (`[n] _ => ...` is rejected), so the port
declares `domain I = 0..n` only for that.

**Coordinate compression leaves a `!` at every use (open, noi16p4).** `let
(X, coord) = compress(ends)` is good; `coord.first_ge(s.l)!` at each lookup
is not, because the value is known to be present. Want `X.of(value)`.

**An ordered pair as an input type (open; ccc21s5, noi16p4).** `where
all(t.l <= t.r for t in reqs)` rejects a third of random cases. A range
read as two numbers with `l <= r` would be generated correctly from the
start.

**A value domain used as a number (open, ccc21s5).** `domain X = 1..=16` is
a value, not an index, and needed `x.label` at each use.

**`Subset<Colour>` cannot be written where a value is expected (open,
btoi18p6).** `for s in Subset<Colour>` fails; the port needs a `type` alias
first.

**Neighbours on a cycle (open, btoi17p3).** `tour[(i + 1) % tour.len]` is
the one raw index in that port. A `cyclic_pairs` view would remove it.
`windows(k)` would do the same for ccc20s3.

**`where` must share a line with `read()` (open, cco16p2).** An indented
continuation is rejected, which makes long lines.

**No early exit at top level (open, noip14p3).** Printing one of two
answers and stopping needed the program wrapped in `fn fly()` so that it
could `return`.

## Bugs found in the prototype

These are fixed unless marked, and each should become a test in the Rust
compiler.

- `if let b = e and cond` parsed `and cond` as part of `e`.
- `sorted(xs)` evaluated `xs` twice.
- A `read()` inside a top-level loop broke the split between input and
  solution, so the generator emitted only the first iteration.
- `a % b` could emit a helper whose definition had been dropped from the
  output.
- A variable named `ends` collided with `std::ends`. The prototype avoids
  C++ names with a deny-list; the Rust compiler must mangle every user name.
- The default generator for a range that crosses zero produced only values
  near the low end, so every weight in cco16p2 was negative and 511 "passing"
  tests tested nothing. Ranges that cross zero now sample around zero.
- Open: the complexity warning fires on a `where` precondition, which is
  compiled out of a submission.
- Open: a runtime trap in a one-line `for i in S: stmt` reports the next
  line number.

## What differential testing found in the corpus

The accepted submission for cco16p2 is wrong when the other points are
collinear with the pivot: its comparator merges opposite directions and
counts them twice (`2 / 0 0 1 / 1 1 5` prints 11; the answer is 6). The
judge's data has no three points on a line, so it passed. The port now
states that precondition (`where general_position(hares)`), and the
generator respects it. Two conclusions: an original is an oracle only
inside the problem's constraints, so each port must declare them; and a
generator that ignores them produces false alarms, while one that is too
narrow produces false confidence (the all-negative weights above).

## Solutions that changed shape

The ports were re-solved, not translated. The cases where the language
pushed towards a different and shorter algorithm:

- **btoi17p3**: a virtual tree per query became "+1, +1, -2 at the LCA" over
  consecutive nodes of the sorted tour, then one pass of subtree sums.
- **noi14p4**: binary lifting over the border tree, O(n log n), became a
  second pointer that never moves back, O(n).
- **cco16p2**: a full-circle sweep with a four-region comparator became one
  sort by cross product over directions folded into the upper half-plane.
- **ceoi18p1**: an in-place knapsack whose loop direction depended on the
  item became copy-then-update, which cannot reuse an item by accident.
- **noip09p3**: a two-layer Dijkstra became four floods in sorted order.
- **dmopc21c2p1**: a sweep over 10^6 positions became a loop over the
  breakpoints of a sorted array.

# Friction report: 76 corpus problems re-solved in `proto/cpc`

Seventy-six accepted C++ submissions from `corpus/` are now re-solved in CP
Lang under `proto/ports/`: 30 from the first two rounds, 8 that the scout
sprint moved up from its sketches, and 38 added in the third round. Every
port compiles, and `proto/run_all.sh` checks each against its original on
random inputs. Line counts, test status and benchmarks per port are in
[`ports-report.md`](../audits/ports-report.md); this file is what the ports taught us
about the language. The third round's findings are in
[their own section](#what-the-third-round-38-more-ports-added); the earlier
sections are unchanged.

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

## What the third round (38 more ports) added

The third round widened the corpus sample on purpose: twelve problems of
the AtCoder DP set, spanning trees and searches, a tree problem, range
structures, strings, integer geometry, number theory, a constructive
problem, two more interactive problems, and a run of older NOIP/CCC/IOI
problems that are short in C++ and therefore a harder baseline to beat.

`main` moved under this round twice. The ergonomics audit landed first, and
the drafts were moved onto it: optional steps (`if let rest = p - a and not
wins[rest]`), `and let` chains, open ranges and `prefix` replaced the raw
index arithmetic the first drafts had (`dpk`, `dpe`, `dpn`, `noip06p2`,
`noip07p2`, `ccc03s4`, `ccc19s5`). Then the scout sprint landed `Dsu`,
`bfs` and `subsets` in the library, which this round had also prototyped,
and thirteen of the problems ported here arrived from sibling sessions
(`ccc18s5`, `ccc21s4`, `ccc22s4`, `cco18p1`, `cco20p1`, `ioi04p4` in
`proto/ports/`; `ccc22s3`, `dpo`, `dpp`, `dps`, `dpu`, `dpv`, `noip06p1` in
`proto/ports/hillclimb_dp_graph/`). Their versions are the ones in the
tree, and the remaining ports use the library's spelling (`use
std.dsu.{Dsu}`, `parts.unite(a, b)`, `bfs(v => adj[v], from: [s])`). What
is listed below is what was still in the way afterwards. Where the scout
pass in [`friction-backlog.md`](friction-backlog.md) lists the same thing,
its item number is given.

### A generated word needs the right characters (fixed; ccc03s4, noip07p2)

`let s: [domain S]Char = read()` generates lowercase letters only, so a
port whose input is digits, or letters and dashes, is "tested" on words
that never reach its interesting branches and passes vacuously, the same
false confidence as the all-negative weights of the earlier round. When
the characters are one range, the element type can be a domain of
characters (from the ergonomics audit). When they are not, the read now
names them. This is test-only information, like `gen:` and `where`:

```
let s: [domain S]Char = read(alphabet: "-abdx-159")     // noip07p2
```

### Input that is not a fixed shape cannot be described (open)

Three kinds of input stopped a port or bent it:

- **Sentinel-terminated and read-to-end input** (`ccc04s5`, `noip99p1`):
  "test cases until `0 0`" and "numbers until end of file" have no `read()`
  form, and the generator cannot produce them. Both problems were skipped.
- **`read()` inside a function generates nothing** (`noip04p1`): the port
  first read its twelve numbers in a function with an early `return`; the
  generator emitted an empty input. Reads must be at top level, which also
  means the early exit had to become a flag. This is a prototype bug, but it
  shows the two open items (no top-level early exit, reads only at top
  level) are one problem.
- **Arrays sized by a loop variable** (`ccc10s4`): `let posts: [m]Post =
  read()` inside a loop works, but a generator function cannot return
  `[m]Post`, so the only way to keep generated polygons valid is a `where`
  that rejects nine inputs in ten. The same limit forces `domain E = 1..n`
  where `[n - 1]T` was meant (`dpg`, `noip14p2`).

The triangle of
`ccc19s5` and `ioi94p1` is read cell by cell into `[Row]List<Int>` (F4).

### No floating point (open; F15)

The prototype has no `Float`. That excluded `dpi`, `dpj`, `ccc05s5` (a
two-decimal average), `cco18p4` (golden-section search) and every geometry
problem that is not exact on integers. The geometry ports of this round
(`cheerio1s3`, `yac6p3`) are integer geometry.
This is the largest hole in corpus coverage.

### Small things that cost a minute each (open unless marked)

Each of these made a port stop and look something up, which is the signal
the porting rule asks for.

- **`var (a, b) = (0, 0)`** declares names that are not `var` (`noip14p2`,
  `noip04p1`). It should either work or be rejected at the declaration.
- **`x += k` on a position** is rejected with "use `i = i + k`", but
  since the audit `i + k` is a `?I`, so the line that compiles is `seen =
  (seen + step)!`. The message is stale, and a counter that walks a domain
  wants `+=` with the same trap as `!`.
- **`prefix(a)` takes `Int` only** (`dpm`): a counting DP sums `Mod` values
  over a window, so it still builds its own `[C.bounds]Z` table.
- **Range bounds are re-evaluated every iteration** (`noip06p2`): `for t in
  0..bundles.len` with a `push` in the body never ends. A `for` range should
  be evaluated once; this one cost a timeout to find.
- **Comprehension gaps**: `sorted(D, by: ...)` over a domain needs `[e for
  e in D]` first and `count(e for e in D if c)` must be `count(c for e in
  D)` (`ccc17s4`); a filter between two `for` clauses is rejected
  (`ccc19s5`); `xs.rev` works in a `for` but not as a value (`dpt`).
- **A tuple literal compared with a named tuple** emits two C++ struct types
  with no `==` between them (`dpy`); binding the literal to a typed `let`
  works. Prototype bug.
- **`Set<(at: Int, stop: V)>()`** does not parse; the tuple needs a `type`
  alias first.
- **A string literal cannot be indexed** (`noip00p1`): `"0123456789AB"[d]`
  had to be `if d < 10: '0' + d else: 'A' + (d - 10)`.

### What the benchmark found (open)

`tools/bench` runs every port against its original on the largest inputs
its generator produces. Two ports were clearly slower, for reasons that
belong here:

- **`Mod` division looks like one operation** (`dpy`, fixed in the port).
  `fact[a + b] / (fact[a] * fact[b])` is a modular inverse per call and
  made the port eight times slower than the original, which keeps inverse
  factorials. The port now does too. A `choose` in the library removes the
  trap; a lint on `/` between `Mod` values inside a loop would catch the
  rest.
- **No hash set** (`ccc03s4`, open). Counting distinct substring hashes
  with `Set<Int>` was 8x slower than the original's open-addressing table;
  sorting each batch is 4x slower and `HashMap<Int, Int>` is worse than
  that. The library needs a flat `HashSet` that can be cleared and reused.

- **One large gap in a port from the scout sprint** (open). `ioi04p4`
  takes 1.5 s where its original takes 0.06 s, the only port more than five
  times slower. It was not written in this round and has not been looked
  into.
- **Smaller gaps, not yet explained** (open). The two `@memo` interval DPs
  `dpl` and `dpn` take two to four times as long as the originals' loops on
  runs of a few hundredths of a second; `coci20c6p3`, `cco20p1` and
  `hillclimb_dp_graph/dpp` are two to three times slower on runs of
  similar length. The other ports are within 30% of their original or
  faster.

### Library pieces the originals had and the ports wrote out (open)

Counted honestly in the stripped baseline, these are where the next library
additions pay: factorial tables and `choose` (`dpy`), a matrix type with
power (`dpr`), a line container for the convex hull trick (`dpz`), a
sliding-window minimum (`dmpg16s5` uses a `SegTree` for what a monotone
deque does), longest increasing subsequence (`noip04p3`), and a way to walk
cyclic neighbours (`ccc10s4`).

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
- Third round: `x /= b` and `x %= b` emitted the C++ operators, which
  truncate, while `x / b` and `x % b` were Euclidean. `noip00p1` (base -2
  conversion) printed a wrong digit string until the compound forms took the
  same path as the binary ones.
- Third round: a `where` on a word read (`let s: [domain S]Char = read()`)
  was parsed and then ignored: a digit-DP draft declared `where Pos.len <=
  10000` and the benchmark still generated a 300,000-digit number that
  overran the original's table.
- Third round: `heap.pop()! + heap.pop()!` crashed the compiler when no
  assignment had been compiled before it (`noip04p2`): the code that orders
  two effects read a field that only an assignment sets.
- Open: a tuple that holds an enum cannot be sorted; the emitted C++ has no
  `<` for the enum. Sorting by a key that leaves the enum out works.
- Third round: `interact.py` treated an input rejected by `where` as a
  conversation and crashed on the empty input (`yac6p3`); it now skips and
  counts rejections like `cp.py stress`.
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

The third round found two more originals that are right only inside
constraints the statement gives and the code never checks. `ccc21s4`
assumes the subway line starts at station 1 and that a swap names two
different stops; `cco20p1` assumes every friend stands inside `l..=r`. A
port of either has to declare that precondition and generate inputs that
meet it.
One original, `ccc10s4`, is marked "not my code" in the corpus; it is kept
as an oracle but its line counts say less about the corpus author's style.

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
- **noip00p1**: alternating floor and ceiling division by the positive base
  became `digits.push(left % base); left /= base` with the negative base
  itself, because Euclidean remainders are already digits.
- **ccc10s4**: a 1000 x 1000 table of pen lists per fence became "sort the
  sides, take `runs` of equal ones": a run of two is a shared fence, a run
  of one faces the outside.
- **noip06p2**: bitmasks over a main item's accessories became a list of
  bundles that doubles once per accessory, then a grouped knapsack.
- **dmpg16s5**: a monotone deque became a minimum query over prefix sums.
- **noip10p3**: a binary search over the answer with a two-colouring of the
  feud graph per step became one pass over the feuds from angriest down,
  with a `Dsu` and one `foe` slot per criminal; the graph is never built.
- **yac6p3**: four corner questions give the extreme `x + y` and `x - y`;
  the port names those four numbers and builds both candidate pairings from
  one `cell(sum, diff)` helper instead of four hand-derived formulas.

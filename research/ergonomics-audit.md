# Ergonomics audit: reading the 30 ports as code

The friction report ([`friction.md`](friction.md)) was written while
porting: it records what was hard to *write*. This audit is the other
half. I read every `.cp` file in `proto/ports/` as a reader would, asked
of each line "is this what the solution means, or is it bookkeeping?", and
changed the prototype wherever the answer was bookkeeping.

It started from two files Ryan flagged, `ccc20s3.cp` (the `shift` helper)
and `coci13c1p3.cp` (four quadrants typed out by hand). Both turned out to
be instances of patterns that recur across the corpus, so every change
below is a general one: nothing was added for a single port, and each
change is used by at least two ports or removes an existing special case.

Each item says where it stands: **fixed** (the prototype changed and the
ports use it), **prototyped** (works in `proto/cpc` as a compiler builtin
that stands in for library code, like `SegTree` does), or **open** (seen
in the ports, not yet changed).

Eleven ports were rewritten. All 30 still agree with their original C++
under `proto/run_all.sh`, and the new vocabulary is locked by
`crates/cpl-cli/tests/cases/ergo/`, `crates/cpl-cli/tests/cases/lend/writeback.cp` and six new
`crates/cpl-cli/tests/cases/bad/` programs.

## The two files that started it

### `ccc20s3.cp`: a sliding window of letter counts

Before, the solution kept two count arrays and a hand-maintained number of
letters on which they disagree. The interesting line of the algorithm
(`if off == 0`) was buried under the code that keeps `off` right:

```cp
let needle: [domain N]Char = read()
let hay: [domain H]Char = read()
domain Letter = 'a'..='z'

var want: [Letter]Int = 0
for c in needle: want[Letter(c)] += 1

var have: [Letter]Int = 0
var off = count(want[c] != 0 for c in Letter)
fn shift(c: Letter, by: Int):
    if have[c] == want[c]: off += 1
    have[c] += by
    if have[c] == want[c]: off -= 1

let hash = StrHash(hay)
var found: Set<Int> = []
for end in H:
    shift(Letter(hay[end]), 1)
    if let start = H.try_at(end.idx - N.len + 1):
        if off == 0: found.insert(hash.of(start, end))
        shift(Letter(hay[start]), -1)
print(found.len)
```

After:

```cp
domain Letter = 'a'..='z'
let needle: [domain N]Letter = read()
let hay: [domain H]Letter = read()

var missing = counts(needle)
let hash = StrHash(hay)
var found: Set<Int> = []
for i in H:
    missing[hay[i]] -= 1
    if let gone = i - N.len: missing[hay[gone]] += 1
    if missing.distinct == 0: found.insert(hash.of((..=i).suffix(N.len)))
print(found.len)
```

Nineteen lines of code became eleven, and four separate pieces of friction
went away:

- `shift` existed only to keep a derived number (`off`) in step with an
  array. A `Counter` does that itself (§1).
- `Letter(c)` converted every character by hand. The input is declared as
  letters instead, so it is checked once, when it is read (§6).
- `H.try_at(end.idx - N.len + 1)` left the typed world to do arithmetic on
  a raw index and came back. `i - N.len` is the same step, typed (§3).
- `hash.of(start, end)` needed both ends of the window to be named. The
  window is one value: the last `N.len` positions up to `i` (§2).

### `coci13c1p3.cp`: rectangles around a grid point

Before, the solution built a 2-D prefix sum by hand and then spelled out
the four quadrants around a point. Each quadrant needed its own mix of
`.l` and `.r`, and getting one of the sixteen wrong still type-checks:

```cp
type B = I.bounds
var pre: [B, B]Int = 0
for r in I:
    for c in I:
        pre[r.r, c.r] = pre[r.r, c.l] + pre[r.l, c.r] - pre[r.l, c.l] + a[r, c]

fn area(r1: B, c1: B, r2: B, c2: B) -> Int:
    pre[r2, c2] - pre[r1, c2] - pre[r2, c1] + pre[r1, c1]

fn matches(xs: List<Int>, ys: List<Int>) -> Int:
    var seen: HashMap<Int, Int> = []
    for x in xs: seen[x] += 1
    sum(seen.get(y) ?? 0 for y in ys)

var total = 0
for i in I:
    for j in I:
        let (r, c) = (i.l, j.l)
        let se = [area(r, c, x.r, y.r) for x in i..=I.last for y in j..=I.last]
        let nw = [area(x.l, y.l, r, c) for x in I.first..i for y in I.first..j]
        let sw = [area(r, y.l, x.r, c) for x in i..=I.last for y in I.first..j]
        let ne = [area(x.l, c, r, y.r) for x in I.first..i for y in j..=I.last]
        total += matches(se, nw) + matches(sw, ne)
print(total)
```

After:

```cp
let pre = prefix(a)

fn matches(xs: List<Int>, ys: List<Int>) -> Int:
    let seen = counts(xs)
    sum(seen[y] for y in ys)

var total = 0
for r in I.bounds:
    for c in I.bounds:
        fn corner(rows: Range<I>, cols: Range<I>) -> List<Int>:
            [pre.sum(span(r, x), span(c, y)) for x in rows for y in cols]
        total += matches(corner(..r, ..c), corner(r.., c..)) + matches(corner(..r, c..), corner(r.., ..c))
print(total)
```

Twenty-four lines of code became fourteen, and the four cases are now one
function called four times. Why `.before` and `.after` did not help: they
answer "which boundary is on this side of a cell", and the solution had to
ask that once per coordinate per quadrant. The questions the solution
actually has are "the rows above this point" and "the rectangle from this
point to that cell", and those now have direct spellings:

- `..r` and `r..` are the cells before and after a boundary (§2).
- `span(r, x)` is the range from boundary `r` to cell `x`, whichever side
  `x` is on, so one expression serves all four quadrants (§2).
- `prefix(a)` and `pre.sum(rows, cols)` replace the hand-written table and
  the inclusion-exclusion formula (§4).
- `counts(xs)` replaces the `HashMap` and its `?? 0` (§1).

## What changed, pattern by pattern

### 1. A count per key, and how many are non-zero (prototyped)

Four ports count occurrences into a map or a per-key array (`ccc20s3`,
`coci13c1p3`, `coci20c6p3`, `noi01p5`), and `ccc20s3` also keeps a second
number in step with the counts. `Counter<K>` is the one type for both:

```cp
var seen: Counter<Int> = []     // every count starts at 0
seen[x] += 1                    // no `?? 0`, no `get`
seen[y]                         // 0 for a key never touched
seen.distinct                   // how many keys have a non-zero count
let c = counts(xs)              // a Counter of a list, an array or a slice
```

A counter keyed by a domain is a flat array; any other key hashes. Counts
may go negative, which is what makes the `missing` counter of `ccc20s3`
work: "the needle minus the window" is zero everywhere exactly when the
window is a rearrangement.

`Counter` is not compiler magic, and `crates/cpl-cli/tests/cases/lend/writeback.cp`
proves it by writing one in `.cp`:

```cp
struct Tally:
    cnt: [V]Int
    distinct: Int

    fn at(mut self, v: V) lends mut Int:
        if cnt[v] != 0: distinct -= 1
        lend mut cnt[v]
        if cnt[v] != 0: distinct += 1
```

The code after the `lend` is the write-back that plan/19 §3.4 describes
and the prototype did not have until now. It runs when the caller is done
with the lent count, so `t.at(v) += 1`, `t.at(v) = 0` and
`bump(mut t.at(v))` all keep `distinct` right with no help from the
caller. This is the general answer to Ryan's `shift`: a derived value that
must follow every write to a container belongs in a projection's
write-back, written once, not in a helper every call site has to remember
to use. The built-in `Counter` is that struct, compiled in because the
prototype has no generic structs yet.

Used by `ccc20s3`, `coci13c1p3` and `noi01p5`.

### 2. Ranges that say which side they are on (fixed)

Ten ranges in three ports named an end of the domain only to say "from
the start" or "to the end": `I.first..i`, `j..=I.last`, `l..=P.last`,
`Mills.first..=t`. A range may now leave that end out:

```cp
for x in ..i        // every position before i
for x in ..=i       // up to and including i
for x in i..        // i and everything after
sum(a[i..])         // the same ranges slice
```

plan/19 §4.1 defines a `Range<I>` as the positions between two
boundaries. The prototype now takes that literally, so either end of a
range may be a boundary, and a boundary splits its domain in two:

```cp
for b in I.bounds:
    total(..b)      // the cells before boundary b
    total(b..)      // the cells after it
```

Two library functions complete the vocabulary:

```cp
span(a, b)          // the smallest range covering a and b, in either order;
                    // each is a position (covered) or a boundary (reached)
r.prefix(k)         // the first k positions of r (all of r if it is shorter)
r.suffix(k)         // the last k
```

`span` is what removes case analysis: `span(r, x)` is right whether cell
`x` is above or below boundary `r`. `(..=i).suffix(k)` is the window of
`k` positions ending at `i`, which is how a sliding window reads when it
is said in words.

A range is a value, so it can be passed. `noi01p5` used to pass two raw
integers and re-test every index against them:

```cp
fn sums(lo: 0..=6, hi: 0..=6) -> List<Int>:
    for t in T:
        if lo <= t.idx and t.idx < hi: ...
sums(0, mid) ... sums(mid, n)
```

and now passes the range:

```cp
fn sums(ts: Range<T>) -> List<Int>:
    for t in ts: ...
sums(..mid) ... sums(mid..)
```

Used by `coci13c1p3`, `ccc20s3`, `noi01p5`, `ceoi09p3`, `ioi05p6` and
`coci20c6p3`.

### 3. A step that can leave the domain is an optional (fixed, revises decision 2.9)

This was the most common friction in the corpus. Six ports took a
position apart with `.idx` or `.label`, did integer arithmetic, tested the
result by hand and rebuilt the position:

```cp
if a.idx >= c and payable[a - c]: payable[a] = true                         // noip18p2
if let b = before[c] and c.idx + d.cores in Spare: best[c + d.cores] ...    // ceoi18p1
next[H(min(h.label + up, m))] min= t + 1                                    // noip14p3
if let t = taps[h] and h.label > down: next[H(h.label - down)] min= t       // noip14p3
if let start = H.try_at(end.idx - N.len + 1): ...                           // ccc20s3
```

Each line states the step twice: once as a guard on raw integers and once
as the real step. The two can disagree, and in the old prototype `a - c`
was a run-time trap if they did.

`i + k` and `i - k` on a position now give a `?I`: `none` when the step
leaves the domain. The guard and the step become one expression, and the
three things a program wants at the edge each have a short form:

```cp
if let b = a - c and payable[b]: payable[a] = true                          // skip
if let b = before[c] and let left = c + d.cores: best[left] max= b + d.gain
next[h + up ?? H.last] min= t + 1                                           // clamp
if let t = taps[h] and let low = h - down: next[low] min= t
if let gone = i - N.len: missing[hay[gone]] += 1
a[(i + 1)!]                                                                 // assert
```

`if let` chains with `and let`, so two steps that must both stay inside
need no nesting. Indexing with an unhandled step is a compile error that
names the three choices:

```
`i + 1` is a ?I: it may step outside I. Bind it with `if let j = i + 1`,
give the edge a value with `??`, or assert with `!`
```

Decision 2.9 made `v + 1` an error and offered `v.offset(k)` for the
optional step. The ports show that the optional step is what `+` should
mean: it is the only arithmetic a position supports, no port reached for
`.offset`, and the operator form is what made the lines above short. The
safety argument of 2.9 is kept, since a step can never silently produce a
position outside the domain. `v.offset(k)` stays as the named form of the
same step (the standard library uses it), `.next` and `.prev` as the names
for one step, and `.cyc(k)` for the wrapping one. Both prototypes agree:
`proto/cpc` compiles the step and `proto/cpi` interprets it the same way.

A boundary has a cell on each side, also optional (plan/19 §4.1):

```cp
b.left      // ?I: the cell before boundary b, none at the first boundary
b.right     // ?I: the cell after it
```

`noi14p4` (KMP borders) used to write `s[S(b.idx)]` for "the character
after a prefix of length b" and `Len(b.idx + 1)` for "one longer". It now
reads `s[b.right!]` and `(b + 1)!`, and its two copies of the border-growing
loop became one `grow` function.

Used by `noip18p2`, `noip14p3`, `ceoi18p1`, `ccc20s3`, `coci20c6p3` and
`noi14p4`.

### 4. Prefix sums (prototyped)

A prefix table is the textbook use of `I.bounds`, and it is always the
same four lines plus an inclusion-exclusion formula that is easy to get
wrong in 2-D. `prefix(a)` builds the table and `sum` reads it with ranges,
so no boundary is ever named:

```cp
let pre = prefix(a)             // a: [I]Int or [I, J]Int
pre.sum(l..=r)                  // 1-D
pre.sum(rows, cols)             // 2-D; an empty range sums to 0
```

In the library this is a struct over `[I.bounds]T` with one method; it
is compiled in for the same reason `Counter` is. Used by `coci13c1p3`.

### 5. `x f= y` for any two-argument function (fixed)

`best min= d` and `best max= d` were two special statements. They are now
one rule: `x f= y` means `x = f(x, y)` for any function `f` of two
arguments, built in or the user's own.

```cp
a[i] lcm= x.label               // was: a[i] = lcm(a[i], x.label)
g gcd= v
best longer= candidate          // fn longer(x: Int, y: Int) -> Int
```

This removes a special case from the language instead of adding one. Used
by `ccc21s5`.

### 6. Reading a word as letters (fixed)

`[domain S]Char = read()` reads a word. Two ports immediately converted
every character with `Letter(c)` to index a per-letter array. The element
type can now be a domain of characters, so the conversion happens once, in
the read, where the range is checked:

```cp
domain Letter = 'a'..='z'
let word: [domain W]Letter = read()
for c in word: letters[c] += 1
```

Used by `ccc20s3` and `coci20c6p3`.

### 7. A function declared inside a loop sees the loop's variables (fixed)

`corner` in `coci13c1p3` closes over the grid point `(r, c)`. The
prototype hoisted every top-level function out of the loop it was written
in, so this compiled to C++ that did not. A function declared inside a
block is now a closure of that block.

## Calls made here that Ryan has not confirmed

These are recorded in `decisions.md` §11 as leaning, not settled.

1. `i + k` and `i - k` return `?I`, which revises decision 2.9 (§3).
2. The name `distinct` for a counter's number of non-zero counts. It reads
   naturally for the common case (distinct values in a window) and slightly
   oddly for a difference of two multisets (`missing.distinct == 0`).
3. `x f= y` accepts any function name, not a fixed list (§5).
4. `..=b` with `b` a boundary is an error; only `..b` is allowed, since a
   boundary is not a position that can be included.
5. A statement that reads the receiver while one of its counts is lent
   (`print(t.at(v), t.distinct)`) is accepted by the prototype and sees the
   state before the write-back. The real checker rejects it, because the
   receiver is lent mutably for the statement.

## Still awkward (open)

These were seen in the same read and not changed, in the order I would
take them.

- **Neighbours.** `for nb in [u.prev, u.next]` (dmopc21c9p4, twice) builds
  a list of optionals to visit two cells. A `u.neighbours` generator, and
  the 4- and 8-neighbour forms for grids, would cover it.
- **Raw indices in generators.** `aac5p2`, `btoi17p3` and `btoi17p6` build
  positions from `.label` arithmetic in `rand(...)` calls (`City(t.label +
  1)`). Typed steps cover half of these; a typed `rand(lo..=hi)` over a
  range would cover the rest.
- **Converting between related domains.** `Size(size)`, `Pairs(made)` and
  `Mills(t - a)` in the DP ports turn a count into a position of a `0..=n`
  domain. Each is a real check, but the same three-token conversion on
  every line suggests that "a count up to n" wants to be usable as an index
  of `0..=n` directly.
- **Graph boilerplate.** Adjacency lists, Kahn's algorithm and multi-source
  Dijkstra are rebuilt by hand in five ports. This is library work
  (`Graph`, `topo_order`, `shortest`) rather than language work.
- **Standard structures.** DSU, sieve, binomials and KMP are each written
  out once in the ports. `noi14p4` in particular is still 24 lines; a
  library `borders(s)` would make it about ten.
- **Prefix of anything.** `aac4p2` keeps a running `lcm` in a `[I.bounds]`
  table by hand. `prefix(a)` only sums; a `scan(a, f)` over any associative
  function is the general form.

# Round 1: indices and ranges

Everything else in the stdlib indexes by these, so they come first.

## The model in one paragraph

A **domain** is a finite, ordered index type. Its members are **positions**,
which are `Ord` in that order (`v < w`, `SortedSet<V>`). (R04.2.23)
An array's axes may also be plain ranges, one per dimension:
`[0..=n, 0..=k]Int` is indexed `dp[i, j]` like a product of domains. (R04.2.25)
Between and around positions sit **cuts**: a domain of n positions has n + 1
cuts, `D.cuts`. A **range** is a contiguous run of positions, and its two
ends are cuts. Arrays are `[D]T`, indexed only by `D`, so a vertex never
indexes an edge array and a cut never indexes a position array.

<!-- given:
  let n = 5
  let l = 3
  let xs = [5, 3, 5]
  domain Row = 0..2
  domain Col = 0..3
  domain Comp = 0..=n
  domain Used = 0..l -->
```cp
domain V = 1..=n                          // positions 1..n
domain Pos = 0..n                         // positions 0..n-1; Pos.cuts is cuts 0..=n
let (domain C, comp) = compress(xs)       // fresh domain from a computation
domain Cell = (Row, Col)                  // product: [Cell]T and [Row, Col]T are one 2-D array type
domain State = (comps: Comp, used: Used)  // named factors: s.comps, State.at((c, u)) -> ?State
enum Dir = N | E | S | W                  // a fieldless enum is a domain too: [Dir]Int
domain Letter = 'a'..='z'                 // so is a Char range: Letter(c), [Letter]Int
```

## Domains

| Form | Meaning | Cost |
|---|---|---|
| `for v in V`, `V.rev` | positions in order or reverse order | \(O(1)\) per step (R04.2.1) |
| `V.count` | number of positions | \(O(1)\) (R04.2.2) |
| `V.all` | every position in walk order, as a `Range<V>`: a member of the domain type, apart from a sequence's `s.all(p)` | \(O(1)\) (R04.2.30) |
| `V.find(f)`, `V.count(f)`, `V.filter(f)`, `V.min(by: f)` | a domain is a `Seq<V>` of its positions in walk order (product domains row-major), so every `Seq` method applies to it; `V.count` with no call is still the number of positions | as the `Seq` method (R04.2.20) |
| `f(V)` where a `Range<V>` is expected | a range-backed domain named as a value is its whole range, `V.first!..=V.last!`, so `fn tally(ls: Range<Pos>)` takes `tally(Pos)` | \(O(1)\) (R04.2.21) |
| `V.first`, `V.last` | `?V`, `None` when `V` is empty; write `V.first!` when you know it isn't | \(O(1)\) (R04.2.3) |
| `V.range` | the whole range, same as `..` (`a.keys` for an array's) | \(O(1)\) (R04.2.4) |
| `V.cuts` | the domain of cuts (type `Cut<V>`): `V.cuts.first` is the start, `.last` the end | \(O(1)\) (R04.2.5) |
| `x in V` | is integer `x` a position | \(O(1)\) (R04.2.6) |
| `V(x)` | integer to position, trapping outside `V` | \(O(1)\) (R04.2.7) |
| `V.at(x)` | integer to position, `?V`: `None` outside `V` | \(O(1)\) (R04.2.8) |
| `P.nth(k)` | a product domain walks row-major (the last factor fastest), so `P.nth(k)` decodes a flat 0-based cell number and `for (r, c) in P` visits rows in order | \(O(1)\) (R04.2.9) |
| `V.try_nth(k)` | the position `k` steps into the walk, `?V`: `None` past either end, as `V.at(x)` is for an integer; `Int` has every integer as a position, so `Int.try_nth(k)` is `Some(k)`, and a cut's `l`/`r`, `v.next(k)` and `v.offset(k)` all bound through it | \(O(1)\) (R04.2.32) |
| `D: Discrete`, `D: Domain` | `Discrete` has `ord`, `nth` and `try_nth`: positions in a walk with no count needed. `Domain(Discrete)` adds `count`, `all`, `range`, `mask` and `from_count`, so it means a counted domain. `Int` is `Discrete`, its integers are positions, but it has no count, so it is not a `Domain`; every declared domain, range type, product, enum and `Cut<V>` of a domain is both. Building a `[D]T` from `D` alone (`[D] v => e`, `var a: [D]T = x`, a list for `[D]T`) needs `D: Domain`. `wrap` and `after` read the count, so they are `Cyclic`'s (`Cyclic: Ordered + Domain`), which every ordered domain has and `Int` lacks | (R04.2.18) |
| `P.at((x, y))`, `P((x, y))` | for a product domain `P = (A, B)`, a tuple of integers to a position: `None` (or a trap) unless every part lies in its factor, so a neighbour that may fall off is `if let q = P.at((r - 1, c))` | \(O(1)\) (R04.2.10) |
| `let (r, c) = p`, `p.0` | a position of a product domain is a tuple of factor positions: tuple patterns and `.0`/`.1` work on it, named factors too, and `g[p]` is the same access as `g[r, c]` (decision 16.5), so a neighbour from `p.adj4` indexes directly | \(O(1)\) (R04.2.10) |
| `v.after`, `v.before` | the pieces `v.r..` and `..v.l` of `v`'s domain, so `for j in v.after` walks the positions after `v`; each is a `Range<D>` and goes wherever one is taken | \(O(1)\) (R04.2.19) |
| `a[p]`, `a[x, y]` | for `P = (A, B)`, `[P]T` and `[A, B]T` are the same type: a `p: P` indexes it whole, its parts index it one by one, and `[P] p => f(p)` or `[A, B] p => f(p)` builds it, `p` the cell. An inline tuple of domains `(A, B)` is the same product domain as a named `P = (A, B)` everywhere a domain goes, so a result keyed by a `(R, C)` vertex takes `comp[r, c]` too (decision 159) | \(O(1)\) (R04.2.11) |
| `Pair<V>` | the unordered pairs `{u, v}` of distinct positions, `V.count * (V.count - 1) / 2` of them, walked by `hi` then `lo` (lower-triangle order); `[Pair<V>]T` is packed, and `d[u, v]` is `d[Pair(u, v)]`, which orders its ends, so `d[u, v] == d[v, u]` (decisions 164, 409: on any `[K]T`, `a[x1, ..., xn]` is `a[K(x1, ..., xn)]` when K's constructor takes `n` arguments, else E0303); `Int(p)` is its rank in that order (`{0, 1}` is 0) and `print(p)` writes the rank (decision 291) | \(O(1)\) (R04.2.12) |
| `V.wrap(x)` | integer to position, Euclidean mod `V.count` (`V.wrap(-1)` is the last position): rings and cyclic shifts | \(O(1)\) (R04.2.13) |
| `if let V(j) = i - 10:` | the pattern form of `V.at`: matches only inside `V` | \(O(1)\) (R04.2.14) |
| `Int(v)` | position to its integer (for `1..=n`, that's 1..n) | free (R04.2.15) |
| `Char(l)` | a position of a `Char` domain to its `Char`; `Int(l)` is its code (99 for `Letter('c')`) and `l.ord` its ordinal (2) | free (R04.2.24) |
| `Int(c)` | cut to its integer: the number of positions before it, 0..=n whatever the base (`1..=n`: `v.l` is v-1, `v.r` is v; `0..n`: v, v+1) | free (R04.2.16) |
| `v.ord` | position to its 0-based ordinal: 0 for the first position whatever the base, the same as `Int(v.l)` | free (R04.2.22) |
| `V.cuts(k)`, `V.cuts.at(k)` | integer to cut, the inverse of `Int(c)`: `k` positions before it whatever the base (`1..=n`: `V.cuts(2)` is between positions 2 and 3); trapping, or `?V.cuts` | \(O(1)\) (R04.2.17) |
| `5` where a `V` or `V.cuts` is expected | integer literal is that position or cut (integer domains; also fills `[d]V.cuts = 0`); outside a static bound it is a compile error | free (R04.2.18) |

**Library domains.** Any struct, in std or user code, becomes a domain by
implementing `Index` with three members (decision 504):

```python
struct Two(a: Int, b: Int)              // the pairs a < b of 0..4, 6 of them

impl Index for Two:
    let count = 6                       // how many positions
    @get fn ord(self) -> Int = a * (7 - a) / 2 + b - a - 1
    fn nth(k: Int) -> Two:              // the inverse of `ord`, for k in 0..count
        var a = 0
        var r = k
        while r >= 3 - a:
            r -= 3 - a
            a += 1
        Two(a, a + 1 + r)

var seen: [Two]Int = 0
for t in Two: seen[t] = t.a * 10 + t.b
print(Two.count, Two.first!.ord, [seen[Two.nth(k)] for k in 0..Two.count])
```

The impl writes `count`, `ord` and `nth` under those names; `count` is sized
like a domain declaration, from constants and `let`s but never a `var`, and
it runs where the type is first used: a fn that is never called, or a branch
not taken, never runs it. The
pre-0.3 names `K`, `to_idx` and `from_idx` are errors that name their
replacement. (R04.2.26)
The type is then a domain like any other: `[Two]E` indexes by `t.ord`,
`for t in Two` and `Two.rev` walk `Two.nth(0)`, `Two.nth(1)`, …, and
`Two.count`, `Two.first`, `Two.last`, every `Seq` method over the walk and a
`D: Domain` bound all work as they do for a core domain. `Two.nth(k)` traps
outside `0..Two.count`, so the impl's `nth` never sees such a `k`. (R04.2.27)
Ordinal order is a walk order, not a successor: `V(x)`, `V.at(x)`, `Int(v)`,
`V.wrap`, `.next`, `.prev`, `.l`, `.r`, cuts, `Range<V>` and position
arithmetic stay with the ordered core domains, and naming one on a library
domain is an error that says so. A library type that wants `Int(x)` or a
printed form writes `impl Into<Int>` (`fn into(self) -> Int`) and `Show`, as
`Subset` does with its mask (decisions 291, 390). An always-on precondition
is an `assert` inside `with checks(asserts: on):` (R09.4.1). (R04.2.28)

Counting and order are two capabilities. `Domain` is counting and positions:
`count`, `nth`, `ord`, `all`, `range`, `mask` and the walk. `Ordered: Domain`
is stepping and order: `at`, `wrap`, `.next`, `.prev`, `.l`, `.r`, `.after`
and `.before`. Every core domain (a count, a range, a `Char` range, an enum,
`Int`, cuts, and a tuple or record of ordered domains) is `Ordered`; a type
that implements `Index` is only a `Domain`, so it never gets an order it does
not have. A generic body that steps bounds `D: Ordered`: under `D: Domain`,
`d.next` is E0203 and the message names `Ordered`. (R04.2.31)

`domain V = e` counts its positions from `e`: an `Int` count, a range of
`Int`s, a list, array or string (one position per element), a struct that
holds one of those, or another domain's value: any value whose type has an
impl of the `@role(counted)` trait (§11.6), std's `Sized`, counts its `len`. Any other type is E0302,
which names it: a domain over `F64`s, `Bool`s or a plain struct has no
positions to count. (R04.2.29)

**Pieces.** Coordinate compression for intervals: `let (domain P, piece) =
D.pieces(cuts: cs)` splits a range at the given cuts into the pieces between them,
with `piece[p]` the `Range<D>` each covers (so `piece[p].len` is its real length)
and `piece.covering(a..=b)` the `Range<P>` that tiles `a..=b`. Inputs are cuts,
pieces are ranges, so the `b + 1` and `- 1` of hand-rolled compression never
appear (decision 162).

## Positions and cuts move the same way

| | neighbour of the same kind | the other kind, either side |
|---|---|---|
| position `v` | `v.prev`, `v.next`: `?V` | `v.l`, `v.r`: its cuts; `v.l(k)`, `v.r(k)`: the far cuts of the `k` positions ending or starting at `v` (R04.3.1) |
| cut `c` | `c.prev`, `c.next`: `?V.cuts` | `c.l`, `c.r`: the positions beside it, `?V` (R04.3.2) |

`v.next(k)`, `v.prev(k)` step k at once, `?V` (cuts too). `c.shift(r)` is
the cuts at distances `r` after cut `c`, clipped to the domain and possibly empty
(chords of length `lo..=hi` from `c` end in `c.shift(lo..=hi)`). `D.mask(xs)` is the
`[D]Bool` marking the positions a sequence yields. `a.ring.step(i, d)` steps `d`
positions of `a` with wraparound (a domain alone: `V.wrap(i.ord + d)`). `.next` is
`.next(k = 1)`: only methods marked `@get` (which promises \(O(1)\)) read without
parentheses, and an `@get` method may take defaulted parameters. A plain
method always needs `()`, so nothing costly hides behind a bare name.

One rule: `.l`/`.r` switch kind (left or right side), `.prev`/`.next` keep it.
The receiver's kind is static, so a mix-up is a type error. `v.l..v.r` is
exactly `v`; a prefix DP reads `dp[v.r] = f(dp[v.l], a[v])`.

Arithmetic on a position gives an `Int`: `v + k`, `v - k` and `v - w`
(two positions of one domain) are the position arithmetic; any other
operator (`*`, `%`, `^`, `k - v`) takes `Int(v)` first, as `Int(v) % 4`. A
position compares with positions of its domain and with integer literals
(`v < 3`); against an `Int` variable it is `Int(v) <= r`. Getting back into
the domain is explicit (`v.next`, `V(x)`) (decision 379). (R04.3.3) Arrays over an anonymous range
(`[0..=n]Int`) take any `Int`, checked, so ordinary DP tables stay plain. Such a range counts its cells from a
number (`[1..=n]`, `[-5..5]`); one starting at a value, `[n..4]Int`, is
E0302: name it as a domain, `domain D = n..4`, then `[D]Int`. (R04.3.4)

## Ranges

### Two kinds of range, one type

`Range<D>` is a contiguous run of a domain `D`. `V.cuts` is a domain like
any other, so there are two kinds you meet in practice, and every method
below is generic over `D`: it means the same thing on both. In expression
position `a..=b` is a range value whatever its bounds (`(0..=1e9).first_true(f)`);
the same text after `:` is a range type (chapter 2).

| | `Range<V>`: over positions | `Range<V.cuts>`: over cuts |
|---|---|---|
| written | `l..r`, `l..=r` with position ends | `j..i`, `j..=i` with cut ends (R04.4.1) |
| elements (`for`, `in`, `len`) | positions | cuts (R04.4.2) |
| typical use | a piece of the array: slice, fold, cost | split points, prefix lengths, answers that may be 0 or n |
| example | `a[l..=r]`, `seg.fold(l..r)` | `for j in ..i`, `for k in a..=b` |

**The rule: a range's elements are the kind of its ends.** Cuts in, cuts out:
`for k in a..=b` walks cuts a through b, `for j in ..i` walks the cuts before i.
A range whose start is past its end is empty, never a trap: `5..=3` and
`(5..=3).rev` walk nothing, so `for c in (p..=x).rev` needs no `p <= x` guard. (R04.4.29, permits)

### One exception, in how a literal is typed: the piece

Cuts `j` and `i` bound a piece of the array: the positions between them. So
a half-open range literal with cut ends, written where a `Range<V>` is
expected, builds that `Range<V>` directly:

<!-- given:
  domain V = 0..5
  let i = V.cuts.last!
  let j = V.cuts.first!
  var dp: [V.cuts]Int = 0
  fn cost(r: Range<V>) -> Int = r.len
  let a = [V].filled(1)
  let pre = a.scan(+)
  let seg = SegTree(+, a) -->
```cp
for j in ..i:                      // ..i: no Range<V> expected, so cuts
    dp[i] min= dp[j] + cost(j..i)  // cost takes Range<V>: j..i is the piece
pre.fold(j..i)                     // same: these take Range<V>
a[j..i]
seg.fold(j..i)
```

This is **not a conversion and there is no implicit `Into`.** It decides what
the literal builds, exactly like `5` becoming a position where a `V` is
expected. Consequences:
- Only the literal `a..b` does it. A stored cut range never becomes a piece:
  `let s = j..i; cost(s)` is a type error whose fix is to write `cost(j..i)`. (R04.4.5)
- Only half-open: `j..=i` on cuts names no piece, so it's an error where a
  `Range<V>` is expected. (R04.4.6)
- It's safe because the two readings are the same set: cut `c` in `j..i`
  stands for the position `c.r`, and the cuts j..i-1 are exactly the left
  edges of the positions between `j` and `i`.

### Open-ended ranges

Leaving out an end means the edge of the other end's domain, so the domain
never has to be named and never has to be non-empty:

| written | means | elements |
|---|---|---|
| `..i` | `D.first..i`, where `i: D` | everything before `i` (R04.4.8) |
| `..=i` | `D.first..=i` | everything up to and including `i` (R04.4.9) |
| `i..` | `i..=D.last` | `i` and everything after it (R04.4.10) |
| `..` | all of the indexed dimension | only inside `[...]`: `grid[r, ..]` (R04.4.11) |

A literal end is a **position** of the indexed domain unless the other end
is a cut (then it is a cut, as in the piece rule above). On `domain V =
1..=n`, `a[..5]` is positions 1..=4 and `a[..=5]` is 1..=5; `a[..V.cuts(5)]`
writes the cut. Integer literals never mean a count of positions.

They follow the ends rule like any range: `for j in ..i` with a cut walks
cuts, and `for j in v.r..` walks cuts too. A half-open open-ended literal
with a cut end is a piece where a `Range<V>` is expected, so `a[..c]` and
`a[c..]` slice. For the positions after `v`, write `for j in v.next..` only
when `v.next` exists, or `for j in a[v.r..].keys`; see Open. `..i` needs no
`!`: `i` existing proves its domain is non-empty. Cost: \(O(1)\) to build, like
any range.

<!-- given:
  domain V = 0..6
  var dp: [V.cuts]Int = 0
  fn cost(r: Range<V>) -> Int = r.len
  let i = V.cuts.last!
  let c = V.cuts.last!
  let a: [V]Int = 1 -->
```python
for j in ..i:                        // every cut before i
    dp[i] min= dp[j] + cost(j..i)
let best = a[..c].max()              // the prefix left of cut c
```

The language's implicit conversions are this closed list, none
user-extensible. Each applies wherever an expected type is known: a
binding or field with a declared type, an argument, a return value, an
assignment, and either operand of `==`, `!=` or a comparison whose other
operand fixes the type. Nothing else converts. (R04.4.22)

1. **`T` to `?T`.** `u != from` with `from: ?V` compares `Some(u)` with
   `from`. Free. (R04.4.23)
2. **A range type to a wider one** (`1..=5` to `Int`, `0..=9` to
   `-5..=100`). Free. Arithmetic on range types yields `Int` (chapter 3),
   so a sum of weights `1..=1e9` is an `Int`, and a path distance or
   `dist: [V]?W` built from one is `Int` unless declared narrower.
   An array's element type does not widen this way: `[V]1..=9` to
   `[V]Int` rewrites every cell, an \(O(n)\) cost a move must not hide
   (R05.2.2), so it is an error; write `xs.map(x => Int(x))` (decision 246).
   Nor does a container narrow into range cells: `List<Int>` given where
   `List<0..100>` is expected, at an initializer, assignment, return,
   argument or a std method that copies cells (`ys.extend(zs)`), would
   check every cell, so it is an error (E0302) unless the source's cells are
   already proven in range (`List<0..50>` moves free); write
   `[x for x in zs]`, which checks each value it stores.
   The other way, an `Int` passed or stored into a range type
   (`f(odd - 1)` with `odd: 0..=100`) is a checked narrowing: in range it is
   free, out of range it traps in a checked build, as any range store does
   (R02.10.2). A constant out of range is instead an error when compiled
   (E0902): a literal, a `const`, a module-level `let` that keeps one, or
   negation, `+`, `-` and `*` of those. A local is never a constant, whatever
   it is known to hold, so it is always the checked narrowing.
   (R04.4.24)
3. **Integer literal to position or cut** of an integer domain (R04.2.18).
   For a domain with a runtime count (`domain V = 1..=n`, `n: 1..=2e5`),
   a literal outside the domain at its largest count is a compile error;
   one inside the domain at its smallest count is proven; one between is
   checked like any unproven position. (R04.4.25)
4. **Cut range to piece:** a literal `j..i` whose ends are cuts, where a
   `Range<V>` is expected, is the piece between them (see R04.4.21).
   (R04.4.26)
5. **Value to array fill:** a value `x: T` where `[D]T` is expected is
   the array over `D` with every element `x`, evaluated once; each element
   is its own copy, as with `[x] * n`:
   `var dist: [V]?Int = None`, `var dp: [0..=n]Int = 0`,
   `var adj: [V]List<V> = List()`. Same as
   `[D].filled(x)`; `[D]` followed by a value without `=>` is a type,
   never a table. (R04.4.27)
6. **`Int` to `Mod<P>`:** an integer (a literal or any `Int` value) where
   a `Mod<P>` is expected is `Mod<P>.from_int(x)`, reduced mod `P`:
   `var ways: [0..=n]Z = 0`, `ways[0] = 1`, `cnt[v] = deg[v]`; an `Int`
   operand of `+`, `-` or `*` beside a `Mod<P>` is expected too, so
   `before[i] + 1` is `Mod<P>` arithmetic. (R04.4.28)

**Integer ranges as domains.** An integer range `lo..hi` (or `lo..=hi`,
or a counted `[n]`, which is `0..n`) used as an answer space or array
dimension is an anonymous domain: it satisfies `Domain`, its positions are
the `Int`s in it, and the cut before position `x` is the `Int` `x`, so its
cuts run `lo..=hi` for `lo..hi`. `(1..=5).bsearch(x => x >= 3)` is `3`,
the first position where `ok` holds, and `Graph.directed` accepts the
counted edges of `read<[m](u: V, v: V)>()`. R04.2.16 (`Int(c)` counts the
positions before `c`) is for declared domains, where the two agree when
the base is 0. (R04.4.30)

A general implicit `Into` is the C++ trap: conversions that don't show and
overloads that turn ambiguous.

### Methods

All of these are on every `Range<D>`. The two right columns spell out what
the generic meaning comes to for each kind.

| Form | On `Range<V>` (positions) | On `Range<V.cuts>` (cuts) | Cost |
|---|---|---|---|
| `r.len`, `r.is_empty`, `x in r` | count of positions; membership of a position | count of cuts; membership of a cut | \(O(1)\) (R04.4.12) |
| `r[x]`, `r.keys`, `r.items` | a range's keys are its own elements: `r[x] == x` for `x in r`, a trap outside it; `r.keys` is `r`; `r.items` walks `(x, x)` | same, over cuts | \(O(1)\) (R04.4.31) |
| `r.nth(k)` | the element `k` steps from `r.first`, 0-based, a trap past the end: the positional read every slice has (R04b.1.4) | same, over cuts | \(O(1)\) (R04.4.32) |
| `for x in r`, `r.rev`, `r.step(k)` | positions | cuts | \(O(1)\) per step (R04.4.13) |
| `r.first`, `r.last` | end positions, `?V` | end cuts, `?V.cuts` | \(O(1)\) (R04.4.14) |
| `r.l`, `r.r` | the bounding cuts, `V.cuts` | the bounds one level up (`V.cuts.cuts`); you rarely want them: use `r.first`/`r.last` | \(O(1)\) (R04.4.15) |
| `r.clip(s)` | intersection | intersection | \(O(1)\) (R04.4.16) |
| `r.halve()` | `?(r.l..r.mid, r.mid..r.r)`, two non-empty halves; `None` when `len <= 1` | same, over cuts | \(O(1)\) (R04.4.17) |
| `r.mid` | middle cut, `V.cuts`, always present (`r.l` when empty) | the middle gap, one level up | \(O(1)\) (R04.4.18) |
| `r.first_true(ok)`, `r.last_true(ok)` | first position where false→true `ok` holds; last where true→false `ok` holds; `?V` | same over cuts, `?V.cuts` | \(O(\log n)\) calls (R04.4.19) |
| `r.bsearch(ok)`, `r.bsearch(ok, last: true)` | the cut where `ok` flips (false→true; with `last:`, true→false), `V.cuts`; always exists | the gap between cuts, one level up; use `first_true` | \(O(\log n)\) calls (R04.4.20) |
| `a[r]`, `pre.fold(r)`, `seg.fold(r)` | the piece `r` | n/a: takes `Range<V>`; a literal `j..i` is the piece | as the API says (R04.4.21) |

**Binary search: which element, or where it flips.** `first_true` and
`last_true` return the element, so the predicate reads like the statement
and is never negated; `None` when no element qualifies (an empty range is
the case with no elements). `bsearch` returns the cut where `ok` turns true,
which always exists, for answers that are boundaries: insertion points,
counts, prefix lengths. They agree: `r.first_true(ok)` is `r.bsearch(ok).r`
and `r.last_true(ok)` is `r.bsearch(ok, last: true).l`.

<!-- given:
  let n = 10
  fn cost(k: Int) -> Int = k * k
  let budget = 50
  let x = 30
  domain V = 0..5
  let pre: [V.cuts]Int = 0
  let s = 0
  let a: [V]Int = 0
  let t = 1 -->
```python
let k = (0..=n).last_true(k => cost(k) <= budget) ?? -1      // largest k that fits
let reach = (0..=n).first_true(k => cost(k) >= x)!           // smallest k that reaches x
let c = V.cuts.range.last_true(c => pre[c] <= s)!              // longest prefix with sum <= s: a cut
let below = Int(a.keys.bsearch(i => a[i] >= t))               // how many of sorted a are < t
```

`ok` must be monotone in the direction the name says: false→true for
`first_true` and `bsearch`, true→false for `last_true` and
`bsearch(ok, last: true)`. Checked builds check that the
probes already made agree with one boundary (every true probe after every
false one, \(O(1)\) per probe), which catches most non-monotone predicates;
they never call `ok` more often than release does (see R09.2.3). Answer spaces that aren't a domain are integer ranges, an
anonymous domain whose positions and cuts are both `Int`:
`(0..=1e18).first_true(t => fits(t))`.

<!-- given:
  domain Pos = 0..6
  fn cross(l: Range<Pos>, r: Range<Pos>) -> Int = 0
  fn cost(k: Pos, m: Pos) -> Int = 0
  var dp: [Pos]Int = 0 -->
```python
fn inversions(r: Range<Pos>) -> Int:          // merge-sort shape
    let (left, right) = r.halve() else: return 0
    inversions(left) + inversions(right) + cross(left, right)

fn solve(rows: Range<Pos>, opts: Range<Pos>): // D&C DP optimisation
    if rows.is_empty: return
    let m = rows.mid.r!
    let best = opts.clip(..m.r).min(by: k => cost(k, m))!
    dp[m] = cost(best, m)
    solve(rows.l..m.l, opts.l..best.r)
    solve(m.r..rows.r, best.l..opts.r)
```

These replace `bsearch_first` and `bsearch_last`. Rejected: only the flip
cut, with `.l`/`.r` picking the side (last-true needs a negated predicate, a
`>` vs `>=` bug source); `bsearch(ok, last: true)` returning an element (a
flag where a name says it better; the flag stays on `bsearch`, where it only
says which way `ok` runs); detecting the direction (ambiguous when `ok` is constant); a
sentinel end when nothing qualifies (looks like a real answer).

## Arrays over domains

<!-- given:
  domain V = 0..5
  domain Pos = 0..5
  domain Row = 0..2
  domain Col = 0..3
  let g = Graph.undirected([(V.first!, V.last!)])
  let grid = [Row, Col].filled(0)
  fn f(x: Int) -> Int = x + 1
  let (l, r) = (Pos.first!, Pos.last!)
  let (i, j) = (Pos.cuts.last!, Pos.cuts.first!) -->
```cp
var dist: [V]?Int = None     // fill
let deg = [V] v => g.deg(v)  // build from a fn
let a: [Pos]Int = read()     // read n values
let pre = a.scan(+)
pre[i] - pre[j]  // prefix sums are indexed by cuts
pre.fold(j..i)
a[l..=r]          // a slice view, O(1), lent
a.keys            // V
a.items           // (V, T)
a.map(f)          // [V]U
grid[Row.first!]  // a row view of [Row, Col]T
```

## Empty domains

Domains may be empty: iteration, arrays and folds over an empty domain just
work. `V.first`/`V.last` are `?V`, so the empty case is visible where you
ask for an end:

```python
let m = read<0..=2e5>()
domain E = 0..m
let e = E.first!             // traps naming `E` when m == 0
let first = E.first ?? return    // or handle it
```

No `NonEmpty` bound: the checker does not track emptiness. Tried and dropped
for now (decision 137); `!` is one character and says the same thing.

## Middle of a range

`r.mid` is the middle cut: ⌊n/2⌋ elements before it (C++'s `(l + r) / 2`
on a half-open range), always present. `r.halve()` is
`(r.l..r.mid, r.mid..r.r)`, `None` when either half would be empty.

| n | `r.halve()` | `r.mid` | `r.mid.r` |
|---|---|---|---|
| 0 | `None` | `r.l` | outside `r` (guard first) (R04.7.1) |
| 1 | `None` | `r.l` | the element (R04.7.2) |
| ≥ 2 | ⌊n/2⌋ and ⌈n/2⌉ elements | `left.r` | `right.first` (R04.7.3) |

`halve` is for the merge-sort shape (stop at one element). The element shape
(handle the middle, recurse around it, stop at empty) is
`if r.is_empty: return` then `r.mid.r!`; lint flags `.mid.r` with no
emptiness guard. Rejected: `r.mid: ?V`, the middle element (safe on empty,
but the cut is what both shapes share, and `halve` reads as its definition).

## Decided (decision 137)

- `.l`/`.r` switch kind, `.prev`/`.next` keep it; `.next(k)`/`.prev(k)` step k. (R04.8.1)
- Position arithmetic gives an `Int`; back into the domain is explicit. (R04.8.2)
- `r.first_true(ok)`/`r.last_true(ok)` return `?` the element; `r.bsearch(ok)`
  returns the flip cut, always present; `last: true` for true→false. They replace `bsearch_first`/`bsearch_last`. (R04.8.3)
- Empty domains are valid; `V.first`/`V.last` are `?V` (no `NonEmpty` bound). (R04.8.4)
- One generic `Range<D>`; elements are the kind of the ends, so `a..=b` on cuts
  is cuts. The half-open literal `j..i` with cut ends builds the piece where a
  `Range<V>` is expected (124): literal typing, not a conversion; no implicit
  `Into`. A position-only reading was rejected: it broke
  `for j in ..i: dp[j] + cost(j..i)`. (R04.8.5)
- `..i`, `..=i`, `i..` are open-ended ranges over the end's domain; bare `..` only as an index or an argument (`a[..]`, `t.fold(..)`), whose place names the domain; elsewhere it is E0302. (R04.8.6)
- `r.mid` is the middle cut, always present; `r.halve()` is `?(r.l..r.mid, r.mid..r.r)`, `None` at length <= 1. (R04.8.7)
- Integer literals convert to positions and cuts of integer domains. (R04.8.8)
- `V(x)` traps outside `V`; `V.at(x)` is `?V`; the pattern `V(j)` refutes. (R04.8.9)
- A list literal for `[D]T` gives one value per cell, so `[]` fills an empty `D`; a list whose length differs from a written count is a compile error, and from a run-time count a trap. A value of the element type is a fill instead: `var out: [D]List<Int> = []` gives every cell an empty list, because `[]` is a `List<Int>` (decision 385). (R04.8.10)
- An array over a domain that does not count its positions (its `Domain` impl gives no `count`; among core domains only `Int`) is the list: `[D]T` with such a `D` is `List<T>`, whose length is the value's own. The rule applies at substitution, so a generic `[D]T` at `D = Int` is `List<T>`. In a body where `D` is a type parameter not bound by `Domain`, `[D]T`'s members are those of the traits both `Array` and `List` implement (`Seq`, `Sized`, `Indexed`, `IndexMut`, ...), and the impl is chosen at the instance, as for any bounded generic. (R04.8.11)

Rejected: `.start`/`.end`/`.before`/`.after` (four words for one idea);
`.n`/`.p` (`n` reads as a count, `p` as parent); `V.first` always trapping
(hides the n = 1 case until a hidden test hits it); `@get`-free bare calls
(silent cost); `V.try(x)` (`try` reads like error handling).
`r.split(at: c)` (`r.l..c` and `c..r.r` say it directly, and also say
"all but `v`": `r.l..v.l`, `v.r..r.r`).

## Why, and the alternatives

This section is rationale, not rules; the rules above decide.

**Why domains.** Off-by-one and wrong-index-space bugs are P0 classes: a
vertex used as an edge index, a 1-based label used as a 0-based position.
Each array over `V` checks its index is a `V`, so one `domain V = 1..=n`
protects every array, loop and read over vertices. The alternatives:

- *Plain `Int` indices* (C++, Python): nothing tells a vertex from an edge
  or a 1-based label from a 0-based one.
- *A newtype per index* (C++ strong typedefs, Rust `struct V(usize)`): it
  keeps spaces apart but loses ranges, cuts, `.next` and reading in base,
  so each program rebuilds them.
- *Dependent or refinement types*: they could prove every index, but need
  analysis well beyond linear time, and "compile speed beats exotic
  analysis".

**Why positions and cuts differ.** "Element `i`" and "the boundary before
`i`" are both `i` in C++, so `pre[i]` against `pre[i + 1]` is a coin flip.
A domain of n positions has n + 1 cuts, and `pre[v]` (through `v`) and
`pre[c]` (left of `c`) are different types (decision 349). On plain `Int`
keys the two are both `Int`, so `pre[k]` is E0320 and a count reads the
table as `pre.cut(k)`, the fold of the first `k` (decision 507). Keeping only half-open `Int` ranges (the Python
and C++ convention) gets the empty range right but leaves every prefix
index to be shifted by hand.

**Why views keep their keys.** `a[l..=r][i]` is `a[i]` (decision 496):
a key names the same element through every slice, filter and sort, so a
position from one view indexes the source. Rebasing each slice to start
at 0 (Python) makes every index shift per view, the off-by-one the domain
removed. The price is that a view whose walk is out of key order, such as
`a.rev`, can't be indexed: `a.rev[0]` is an error and `a.rev.nth(0)` reads
by walk position.

**Rejected names** are listed under "Decided (decision 137)".

## Open

None. (The positions after `v` are `v.after`, decision 186.)

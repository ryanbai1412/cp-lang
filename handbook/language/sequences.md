# Sequences

| | |
|---|---|
| Kind | Language design |
| Owner | spec |
| Leads | `spec/core/07-sequences.md`, `spec/lib/sequence-patterns.md` |
| Describes | `spec/core/07-sequences.md`, `spec/lib/sequence-patterns.md`, `spec/lib/ops-ranges.md`, `spec/lib/search-dp.md`, `std/seq.cp`, `spec/conformance/core-07/` |

Why walking, slicing, folding and searching sequences work the way they do
in CP Lang. A *key* is the value that indexes a sequence, a *cut* is a gap
between positions, and a *view* is a sequence made from a source without
copying it (`spec/GLOSSARY.md`, `spec/core/07-sequences.md` "Terms").

## The problem

Sequence code fails at the joints: where a subarray loses track of where
it came from, where a reversed walk is read by index, where a helper
answers `-1`. Three examples:

```cpp
// 1. The argmax of a subarray is relative to the subarray.
auto it = max_element(a.begin() + l, a.begin() + r + 1);
int i = it - a.begin() - l;      // relative to l; forgetting "- l" or adding it twice is a WA
// 2. Prefix sums: is pre[i] the sum through i, or the sum before i?
long long s = pre[r] - pre[l];    // off by one in half the codebases you copy from
// 3. Sentinels: an empty input returns INT_MIN, which then gets added to something.
```

```python
# 4. Slices copy, and generators are one-shot.
best = max(a[l:r+1])              # O(n) copy every call; in a loop this is the TLE
g = (x * x for x in a)
print(sum(g), max(g))             # ValueError: max() arg is an empty sequence
```

C++20 ranges solve the copy, but `views::reverse | views::filter` has
a different iterator category at each step, and a reversed view is still
indexable by walk position, which is easy to confuse with the source index.

## The model

<!-- given:
  domain I = 0..6
  let a: [I]Int = [I] i => Int(i) % 3
  let (l, r, k) = (I(1), I(4), 3) -->
```cp
let w = a[l..=r]                  // a view: O(1), keeps a's keys
let i = w.keys.max(by: k => a[k])! // a key of a, usable as a[i]
let pre = a.scan(+)                // pre[c] folds left of cut c
for x in a.rev: print(x)           // walks backwards; a.rev[k] is an error, a.rev.nth(k) isn't
let longest = a.maximal(Window(+), ok: win => (win.all ?? 0) <= k).max(by: p => p.len)!
```

What a contestant holds in their head:

1. **A view keeps its parent's keys.** A slice, a filter, a reversed walk
   all answer in the source's keys: `a[l..=r][i] == a[i]`. Only
   materializing (`List(...)`, `.sorted()`, `.collect()`, a comprehension)
   makes fresh keys from 0.
2. **Six capabilities, one question each.** Can I walk it (`Seq`)? Walk with
   keys (`Keyed`)? Know its length in \(O(1)\) (`Sized`)? Walk it backwards
   (`Bidir`)? Read `s[k]` in \(O(1)\) (`Indexed`)? Write `s[k]`
   (`IndexedMut`)? `filter`, `take_while`, `.rev` and generators are
   walk-only: no `s[k]`.
3. **Folds are methods on anything walkable.** `sum`, `max`, `count`,
   `fold`, `any`, `find` take a generator, a filter or a list, and the
   receiver's type picks the algorithm (a range sums in closed form).
4. **Scans are indexed by cuts.** `pre.cut(k)` is the fold of the first
   `k` elements; a bare `Int` index `pre[k]` is an error (E0320).
5. **Pieces, then folds.** Windows, runs, splits and two-pointer spans are
   all "cut the sequence into keyed pieces, then fold each piece".

## Key decisions

### One iterator: the generator (R07.1.1, decision 126)

**Chosen.** A walk-only sequence is a generator (`-> yields T`), and
library loops are generators, so they compile to the loop you would write
by hand and keep `break` and `return` working (see
[control-and-functions.md](control-and-functions.md)). A chain is a recipe
that reruns on every walk (R07.1.3).

**Lost.** Pull iterators with `next()`: each recursive library traversal
would need an explicit stack. Python's one-shot generators, whose second
walk is silently empty. The cost is no lockstep over two generators;
`List(g)` materializes one, visibly (R07.6.4).

### Six capabilities instead of one iterator trait (decision 142)

**Chosen.** `Seq`, `Keyed`, `Sized`, `Bidir`, `Indexed`, `IndexedMut`
(R07.3.1 to R07.3.6). `Indexed` is the cost line: it promises \(O(1)\)
`s[k]`, `s.keys` and `s.len`. A filter cannot keep that promise without
a hidden \(O(n)\) skip, so `filter` is `Keyed` but not `Indexed`, and
`w[k]` on it is a compile error rather than a silent linear scan.

**Lost.** One `Iterator` trait (Rust, Python): every fold works, but a
function that needs \(O(1)\) reads can't say so, and `.nth(k)` on a filter
is a hidden \(O(n)\) in a loop. C++'s five iterator categories: the right
idea, but categories describe iterators rather than the sequence, so a
contestant writes `std::next(it, k)` and gets \(O(k)\) without a warning.

**Every fold still takes walk-only input.** `a.filter(p).sum()` and
`gen(n).max()` work, because folds need only `Seq` (R07.1.2). The split
costs nothing where it doesn't matter and errors where it does.

### The sequence vocabulary is std code (decision 509, R07.2.18)

**Chosen.** `Seq`, `Indexed`, `Bidir`, the generator value `Gen<T>` and
every adapter, `.rev` included, are ordinary declarations in
`std/seq.cp`. `fn total(s: Seq<Int>)` takes a generator, a list or a
reversed view because each is a std type implementing a std trait, and
`.rev` on a generator is the plain bound error "requires `Bidir`".

**Lost.** Compiler-known sequence types, which let the checker special-case
`.rev` and generators without a library declaration, at the price of a
second, invisible definition that user code could not read or extend.

### Identity everywhere: keys in, keys out (decision 143, R07.1.4)

**Chosen.** Every slice keeps its source's keys, and library functions
answer in the keys they were given. `s[2..5][3]` is `s[3]`, and
`s[2..].search('a')` returns a key of `s`. Relative access has its own
words: `w.first`, `w.nth(k)` and `w.rebase` (a view counting from 0).
`.items` on a filter yields the source keys of the survivors.

**Lost.** Python, C++ and Rust slices re-index from 0. That reading is
familiar, which is why decision 143 weighed it against the bug it causes:
every function called on a subrange returns a relative answer that the
caller must shift back. With source keys the answer is usable as
`a[i]` directly, and a domain-typed array makes a leftover relative index
a type error (`spec/core/04-indices.md`). Writing `w[3]` with a literal on
a slice gets lint L0605, which says the key is the source's and names
`w.nth(3)`.

### Keys never wrap; the last element is `a.last!` (decision 508, R07.2.17)

**Chosen.** An `Int` key counts from the first element and nothing else.
`a[-1]` with a negative literal is error E0321, whose fix is `a.last!`;
the `k`th from the end is `a[a.len - k]`. A computed negative key traps
R0101 like any key out of range.

**Lost.** Python's `a[-1]`. It is short and familiar, but in a language
whose keys can be computed it turns an off-by-one below zero into a quiet
read from the far end, and the C++ it compiles to has no such meaning.

### One indexing rule for every keyed view; `.rev` is walk-only (decision 496, decision 388)

**Chosen.** Only a view that walks in key order is indexable, and
indexing always means "by source key" (R07.5.3). `.rev` walks out of key
order, so `w.rev[k]` is error E0305, whose fix offers `w.rev.nth(k)`
(the `k`th in walk order) or `w[k]` (the key). `.rev.items` and
`.rev.keys` keep the source's keys (R07.4.7).

**Lost.** Both obvious readings of `a.rev[k]`. "The `k`th from the end"
is what C++ and NumPy do; "the element at source key `k`" is what
identity-everywhere suggests. Both looked right in code review, and one
was always silently wrong, so neither is the meaning. `zip(a, a.rev)`
still pairs each element with its mirror, by walk position (R07.6.4).

The same rule settles a literal (decision 388): `w[x]` with `x` proven
to be one of `w.keys` reads with no check, a literal that is not proven a
key is a compile error naming `.nth`, and any other `x` is checked and
traps naming `w.keys`. In review, `a.rev[0]` and `r[0]` on a run were
read as "the first element" every time, so the literal case is rejected
before it can trap (R07.4.5):

<!-- rejects: E0305 -->
```cp
let a = [10, 20, 30, 40]
let w = a.rev
print(w.nth(0), w.first!)  // 40 40: positional on any view
print(w[0])                // error E0305: write `w.nth(0)`, or `a[0]` for key 0
```

### Laziness is explicit; allocation has a name (R07.1.3, R07.1.7, decision 140)

**Chosen.** Views are \(O(1)\) to make and allocate nothing; anything that
builds a container says so: a comprehension, `List(s)`, `.collect()`, or
a method named for its result (`sorted`, `tally`, `scan`). In-place verbs
take a `var` (`a.sort()`), and each has a past-participle copying twin
(`a.sorted()`). \(O(1)\) reads are getters without parentheses (`.len`,
`.first`, `.keys`); "parentheses mean work" (decision 140).

**Lost.** Rust's `iter().map().collect::<Vec<_>>()` turbofish and
Python's eager `sorted(...)` versus lazy `map(...)` that differ by
function. A lint flags a chain walked twice, which is where laziness
costs time.

`List(s)` is a conversion, so it takes exactly one sequence and copies
it; a list of given values is the literal `[a, b]`, and a list holding
one list is `[xs]` (decision 348; R07.3.2):

```cp
let xs = [1, 2, 3]
let a = List(xs)    // a copy of xs: 3 elements
let b = [xs]        // one element, the list xs
let c = List(0..4)  // any sequence: 4 elements
print(a.len, b.len, c.len)
```

The variadic `List(a, b)` lost because `List(xs)` would then mean "copy
`xs`" or "a list holding `xs`" depending on the argument's type, and the
literal already says the second.

### Folds are methods; their cost comes from the receiver (decisions 53, 326, 378)

**Chosen.** `xs.sum()`, `xs.max(by: f)`, `xs.fold(op)` are methods
(decision 53), so completion finds them after a dot. Each fold resolves
through ordinary method lookup on the receiver, so `Range.sum` is a
closed form and `(0..3e9).sum()` is \(O(1)\) (decision 486). Folds of a
possibly empty input return an option (`max() -> ?T`); there are no
sentinels. `count()` counts elements; `count(p)` counts matches
(decision 378). Options are named: `max(by: f)`, never `max_by`
(decision 60); descending order is `sorted(by: x => desc(x))`, with no
`desc:` flag (decision 366).

**Lost.** Free functions (`sum(xs)`), which hide from completion.
A compiler pass that recognises `(a..b).step(k).sum()`: it would be a
privilege the stdlib has and a user type doesn't. Returning
`INT_MIN` for an empty max.

**A summary folds as map, then `+` (decision 472).** Kadane's best
subarray is `a.map(MaxSub.of).fold(+).best`: build one summary per
element, then join them with the summary's `+`. `MaxSub.of` is an ordinary
fn value, like any named fn. Writing `a.fold(MaxSub)` would need a type
name to stand for its `of` and `join` pair, a new rule that only summary
types would use. The map spelling needs nothing new, and the same `+` is
what a `SegTree` over `MaxSub` calls.

### Scans are indexed by cuts (decision 349, R07.1.6)

**Chosen.** A prefix scan is a table over cuts: `pre[c]` folds everything
left of cut `c`, and for a domain position `v`, `pre[v]` folds through
`v` (`pre[v.r]`). On `Int` keys a position and a cut are both `Int`, so
`pre[k]` is an error (E0320, decision 507) and `pre.cut(k)` names the
fold of the first `k`. A domain keeps the two
apart, and a sum over `l..=r` is `pre[r.r] - pre[l.l]` with no `+ 1`
to forget. `a.rev.scan(+)` is the suffix table, also indexed by cut,
and the fold of everything but `i` is `op(pre[i.l], suf[i.r])` over the
two; they are the only spellings (decision 511), and `cpl fix` rewrites
the old `suffix_fold(op, a)` and `a.prefix_excl(op)`.

**Lost.** `pre[k]` on a `List`, the shortest spelling of the commonest
read, because the reader of `pre[i]` can't tell a position from a cut.
Inclusive scans (`std::partial_sum`, `itertools.accumulate`),
which make every range sum `pre[r] - pre[l - 1]` with a special case at
0, and the "pad with a leading zero" convention that every codebase
spells differently.

### Pieces, then folds (`spec/lib/sequence-patterns.md` §1)

**Chosen.** One general model replaces a shelf of named helpers: a
mechanism cuts a sequence into keyed pieces (`windows(k)`, `chunks(k)`,
`runs(by:)`, `split(p)`, `maximal`, `minimal`, `spans`), and any fold
runs on each piece. Each piece is an `Indexed` view keeping the source's
keys, so `r.keys.first!` is where a run starts in the source. Folding
`windows(k)` is a sliding fold, \(O(1)\) amortized per window whatever the
operation, with the strategy picked from the operation's laws: a group
(`+`, `^`) adds the new element and subtracts the old, a selective
operation (`min`, `max`) uses a monotone deque, and any other associative
operation (`gcd`, matrix product) uses two stacks. The contestant writes
`fold(+)`, `fold(min)` or `fold(gcd)` and never picks the algorithm.

**Lost.** A named helper per pattern (`sliding_max`, `sliding_sum`,
`run_ranges`, `chunk_by`, `excl_fold`): the older surface of
`std.seq` had all of these, and each was one more name with its own
argument order. Allocating `List<List<T>>` for chunks, which Python's
`itertools.groupby` and many C++ templates do.

### `maximal` and `minimal`: two pointers as a fold over pieces (sequence-patterns §2)

**Chosen.** `a.maximal(state, ok:)` yields, for every right end, the
longest piece ending there that satisfies `ok`; `a.minimal(state, ok:)`
yields the shortest. The state is a `Track`, something with `push` and
`pop` (a `Window(+)` fold, a `Counter`), and the library moves the two
pointers. The law is stated with the call: `maximal` needs a hereditary
predicate (true on a piece means true on every sub-piece), `minimal`
needs an upward-closed one (true on a piece means true on every
super-piece). The promise is checked where it is cheap: `cpl stress`
compares `maximal` and `minimal` with their \(O(n^2)\) definition on small
inputs, so a predicate that breaks the law shows up as a counterexample
naming the piece. Mo's algorithm takes the same `Track` protocol.

**Lost.** Hand-written two pointers, where the common bugs are popping
before the check, forgetting the empty window, and moving the left
pointer past the right. Pairing `maximal` with a monotone deque type the
contestant must pick: the `Track` trait lets a user type play the same
role.

### Search on an answer space is a range method (absorbed from the search rationale)

**Chosen.** "The first `t` where a monotone check flips" is
`(lo..=hi).first_true(ok)`, its mirror `last_true`, and
`r.bsearch(ok)`, which returns the cut where `ok` flips and always
exists (R04.4.19, R04.4.20). The answer is a position of the searched
range's own index space (a `V`, a cut, an `Int`), and `None` when no
element passes, never a sentinel (decision 377). Checked builds probe the
ends and catch most non-monotone predicates. The rest of the family is
also range methods in `spec/lib/search-dp.md` "Search": `r.peak(by: f)`
and `r.valley(by: f)` for unimodal functions, `F64` ends for real
searches, `first_true(ok, near: c)` for galloping, and
`parallel_bsearch` for many queries over one timeline. Each states its
law (monotone, strictly unimodal) and its exact number of `ok` calls, so
an interactive problem's query budget can be counted.

**Lost.** `while (lo < hi)` by hand: 28 ports wrote one before the range
methods existed, usually because the answer space was `0..=2e18`, real,
or needed scratch state. A free `bsearch_first(r, ok)`: one name per
search shape, removed for the range method so the space carries the
type (conformance case `core-04/bsearch-first-removed.cp`).

### Range structures share one vocabulary and state their laws (absorbed from the ranges rationale)

**Chosen.** Prefix tables, Fenwick trees, segment trees, lazy and sparse
tables are ordinary library code with one vocabulary
(`spec/lib/ops-ranges.md` §7): `S(op, a)` builds, `s[i]` reads,
`s[i] = x` and `s[i] += y` write through a place, `s.fold(r)` folds a
`Range<D>`, `s.prefix(c)` folds before a cut, `s.apply(r, f)` applies a
lazy tag, `s.right_while(from: c, ok)` and `s.left_while(to: c, ok)`
descend the tree and return cuts like `bsearch`, and `s.all` is an
\(O(1)\) getter. A structure has a method only when its operation supports
it. The operation is an operation value (`+`,
`max`, a monoid), not a wrapper type. The type name picks the algorithm:
`SparseTable` needs idempotence, `DisjointSparseTable` does not,
`Fenwick` needs a group. Each structure documents the law its operation
must satisfy, and `laws_hold` checks it in `cpl test`
(see [traits-and-generics.md](traits-and-generics.md)). The precise
surface is `spec/lib/ops-ranges.md`.

**Lost.** Per-structure method names (`query`, `prod`, `sum`, `get_range`),
which the AtCoder Library and KACTL each spell differently, and the
AtCoder names `max_right` / `min_left`, which `right_while` /
`left_while` replace so the name says which way the walk goes.
`std::function` inside the structures: monomorphized operation values
recover a 1.6x to 4.4x gap measured against hand-rolled C++ in the ports.
Raw `Int` coordinates on fixed-size structures: they take `Range<D>` and
`Cut<D>`, so an index from another domain is a compile error.

## Worked examples

A slice keeps its parent's keys; a filter keeps them through `.items`;
`.rev` reads by walk position with `.nth`:

```cp
let s = [10, 11, 12, 13, 14, 15]
let w = s[2..5]
print(w.first!, w.nth(1))
let best = w.keys.max(by: i => s[i])!
print(best, s[best])
for (i, x) in s.filter(x => x % 2 == 1).items: print(i, x)
print(s.rev.nth(0))
print("abcab"[2..].search('a')!)
```

Scans over cuts, with a domain and with a `List`:

```cp
domain V = 1..=4
let a = [V] v => Int(v)
let pre = a.scan(+)
let v = V(3)
print(pre[v], pre[v.l], pre[v.r])
let p = [3, 1, 4].scan(+)
print(p.cut(0), p.cut(2), p.cut(3))
```

Folds on generators and filters, and a range's own closed-form sum:

```cp
fn sq(n: Int) -> yields Int:
    for i in 1..=n: yield i * i
print(sq(4).sum(), sq(4).max()!, sq(4).count(x => x % 2 == 0))
print([3, 1, 2].filter(x => x > 1).fold(+)!)
print((0..3e9).sum(), (3..4e9).step(2).sum())
```

Runs are pieces that keep source keys:

```cp
let a = [1, 1, 2, 2, 2, 1]
for r in a.runs(by: x => x): print(r.keys.first!, r.len)
```

`maximal`: the longest piece with sum at most `k`:

```cp
let a = [2, 1, 3, 1, 1, 4]
let k = 4
let longest = a.maximal(Window(+), ok: win => (win.all ?? 0) <= k).max(by: p => p.len)!
print(longest.len, a[longest].sum())
```

Search on an answer space, and where a sorted list crosses a threshold:

```cp
let reach = (0..=100).first_true(x => x * x >= 50)!
let a = [1, 3, 3, 7]
let below = Int(a.keys.bsearch(i => a[i] >= 3))
print(reach, below)
```

# Reference

## Where the precise rules are

| Topic | Spec |
|---|---|
| Key decisions | `spec/core/07-sequences.md` "Key decisions", R07.1.1 to R07.1.7 |
| Terms (view, adapter, key) | "Terms", R07.2.1 to R07.2.16 |
| The six capabilities | "The capabilities", R07.3.1 to R07.3.6 |
| Which keys survive a slice; `.rev` | "Slices: which keys survive", R07.4.1 to R07.4.13 (R07.4.7 for `.rev`) |
| View structs and costs; `w.rev[i]` is E0305 | "Under the hood", R07.5.1 to R07.5.6 |
| Generators as sequences; `zip` with `.rev` | "Generators", R07.6.1 to R07.6.4 |
| Folds, scans, ordering | "Folds", "Scans are prefix sums over cuts", "Ordering and grouping", R07.8.1, R07.9.1 to R07.9.6, R07.11.1 to R07.11.5 |
| Pieces, sliding folds, `maximal`, `minimal`, `spans`, `a.ring` | `spec/lib/sequence-patterns.md` §1 to §4 |
| Binary search on ranges | `spec/core/04-indices.md` R04.4.19, R04.4.20 |
| Answer-space search, DP toolkit | `spec/lib/search-dp.md` |
| Range structures and their laws | `spec/lib/ops-ranges.md` |
| Conformance cases | `spec/conformance/core-07/` |

## Changes

- 2026-11-23: first version.

# Round 2: sequences and views

Built on round 1: arrays are `[D]T`, ranges are runs of a domain with cut
ends, `.first`/`.last` are optional.

## Key decisions

1. **Generators are the only iterator.** `-> yields T`, no `next()`; values
   are pushed into the loop body, so they compile to the hand-written loop.
   `break`/`return` work through them; lockstep is `List(g)`. (R07.1.1)
2. **Six capabilities.** `Seq` walks; `Keyed` walks `(key, x)`; `Sized` has
   \(O(1)\) `.len`; `Bidir` walks backwards; `Indexed` adds \(O(1)\) `s[i]` and
   `.keys`; `IndexedMut` makes `s[i]` a place (POWERS W4). Adapters keep `Indexed` when they can stay \(O(1)\) (slices,
   `map`, `take`/`drop`, `zip`); `filter`, `take_while` and generators are
   `Seq`, so every fold (`sum`, `max`, `count`, ...) takes them, `yields`
   members such as `c.values` included. (R07.1.2)
3. **Laziness is explicit.** A chain reruns on every walk. Allocation happens
   only at `.collect()`, a comprehension or `List(...)`. Lint flags a chain
   walked twice (sharper when it holds a `mut` lend) and side-effecting
   closures. (R07.1.3)
4. **Identity everywhere.** Every slice keeps its source's keys
   (`a[l..=r][i] == a[i]`, `s[2..5][3] == s[3]`), and library functions
   answer in the keys they were given. `first`, `nth(k)` and `rebase` cover
   relative access. (R07.1.4)
5. **Views are lent values with the rules of `mut a[i]`.** A value built
   from a lent part is lent; no lifetime annotations. It is kept only
   where R05.7.8 to R05.7.10 allow (storage that dies before its sources)
   and returned only by a fn declared `lends`. In-place verbs
   work on views (`a[l..=r].sort()`); each has a copying twin (`sorted`). (R07.1.5)
6. **Prefix scans are indexed by cuts.** `pre[c]` folds left of `c`;
   `pre[v]` folds through `v`. A bare `Int` index into a cut table
   (`pre[k]` on a `List`, `[n]T` or `[0..n]T` scan, a `Prefix<T>`) is
   E0320: there a position and a cut are both `Int`, so the index can't say
   which it means. A count reads the table by name: `pre.cut(k)` folds the
   first `k` on every key type (decisions 349, 507). (R07.1.6)
7. **Costs show.** \(O(1)\) things are getters or views, allocation is named,
   options are named arguments (`max(by:)`), and there is no overloading. (R07.1.7)

## Terms

Every word below means one thing in this doc, in the stubs and in errors.

| Term | Means | Owns its elements? | Examples |
|---|---|---|---|
| **sequence** (`Seq<T>`) | anything a `for` can walk, in order | either | everything in this table (R07.2.1) |
| **indexed sequence** (`Indexed<K, T>`) | a sequence that also has \(O(1)\) `s[k]`, `.keys` and `.len` | either | `[V]T`, `List<T>`, `Str`, ranges, slices (R07.2.2) |
| **key** | what an indexed sequence is indexed by: a domain position (`V`) or an `Int` offset (`List`, `Str`) | | `i: V`, `3` (R07.2.3) |
| **collection** | a sequence that owns its elements; making one allocates | yes | `[V]T` arrays, `List`, `Str`, `Set`, `Map` (R07.2.4) |
| **range** | a run of a domain, `l..r`; a sequence of positions or cuts with no data behind it (round 1) | nothing to own | `l..=r`, `..i`, `V.range` (R07.2.5) |
| **view** | a sequence made from a source without copying it: \(O(1)\) to make, borrows the source, does its work as you walk or read it. Formally, a `Seq` built from a lent source, so it is a lent value (POWERS W4); there is no `View` trait and no marker on the struct | no | `a[l..=r]`, `a.rev`, `a.map(f)`, `a.filter(p)` (R07.2.6) |
| **slice** | the view `a[r]`: one contiguous range `r` of an indexed source | no | `a[l..=r]`, `s[i..]`, `grid[r]` (a row) (R07.2.7) |
| **adapter** | a method that turns a sequence into a view | | `.map`, `.filter`, `.rev`, `.take`, `zip` (R07.2.8) |
| **chain** | adapters applied one after another; still a view, still lazy | no | `a.filter(p).map(f)` (R07.2.9) |
| **generator** | a fn declared `-> yields T`; calling it gives a sequence that reruns the body on each walk | no | `divisors(n)`, `dfs(g, from: [s])` (R07.2.10) |
| **materialize** | copy a sequence into a new collection: the only allocation step | | `.collect()`, `List(s)`, `[f(x) for x in s]` (R07.2.11) |
| **place** | something assignable: an element or a `mut` view | | `a[i]`, `mut a[l..r]` (R07.2.12) |

How they relate:
- **Two capabilities.** A view is indexed when it walks in key order and
  can answer `w[k]` in \(O(1)\): slices, `.map`, `.items`, `zip`,
  `take`/`drop`. `.rev` reads `w.nth(k)` in \(O(1)\) but walks against its
  keys, so it is not indexed. Everything else is only a sequence
  (walk-only): `filter`, `take_while`, `chain`, generators. (R07.2.13)
- **Slices are the views that narrow keys.** Every other indexed view keeps
  its source's keys unchanged. (R07.2.14)
- **Ranges are indexed sequences but not views:** they have no source.
  Slicing *with* a range gives a view. (R07.2.15)
- **Lazy means a view or generator.** Its work happens at each walk or
  read, never up front. Materializing is the only step that does the work
  once and keeps the result. (R07.2.16)
- **Keys never wrap.** An `Int` key counts from the first element, so
  `a[-1]` with a negative literal key is E0321, whose fix is `a.last!`
  (`a[a.len - k]` for the `k`-th from the end, `V.wrap(-1)` on a domain).
  A computed negative key traps R0101 like any key out of range
  (decision 508). (R07.2.17)
- **No built-in sequence names.** `Seq`, `Indexed`, `Bidir`, a generator
  call's value `Gen<T>` and every adapter, `.rev`'s `Rev<S>` included, are
  declarations in `std/seq.cp` with real bodies. The checker reaches a
  `for`'s walk, a generator's type and an adapter through those
  declarations, as it reaches any library method, and names none of them
  itself; a program names each in a signature through the prelude
  (decision 509). (R07.2.18)

<!-- spec-only: expression tour, one form per line -->
```python
a[l..=r]          // view of a over l..=r; a[l..=r][i] is a[i], i must be in l..=r
s[2..5]           // keeps keys: s[2..5][3] is s[3]; s[2..5].nth(0) is s[2]
a[l..=r].keys     // l..=r
a.rev             // walks a backwards; a.rev.nth(0) is a's last, a.rev[i] is an error
a.map(f)          // Indexed<D, U>: (a.map(f))[i] is f(a[i]), O(1) per read
a.items           // (i, a[i]) pairs with a's own indices
```

## The capabilities

| Capability | What it guarantees | Examples |
|---|---|---|
| `Seq<T>` | `for x in s` walks it in order; each `for` runs it from the start | generators, `s.filter(p)`, `chain(a, b)`, `g[u]` (R07.3.1) |
| `Keyed<D, T>` | `Seq` + `s.items` walks `(key, x)`; keys survive `filter` | everything `Indexed`, `s.rev`, `s.filter(p)`, `s.take_while(p)`, `Map`, `g.keyed()`, `g.keyed(by: f)` (R07.3.2) |
| `Sized` | \(O(1)\) `s.len` | collections, ranges, `Indexed` views, sets, maps (R07.3.3) |
| `Bidir<T>` | `s.rev` walks backwards; `s.last` is \(O(1)\) | everything `Indexed`, sorted sets, deques, `filter` over a `Bidir` (R07.3.4) |
| `Indexed<D, T>` | `Keyed` + `Sized` + `Bidir` + `s[i]` for `i: D` in \(O(1)\), `s.keys: Range<D>` | `[D]T`, `List<T>` and `Str` (`D = Int`), ranges, slices (`a[r]`, `take`/`drop`, `windows`, `chunks`, `runs`, `split`), `.map(f)`, `zip(a, b)`; not `.rev` or `filter` (R07.3.5) |
| `IndexedMut<D, T>` | `Indexed` + `s[i]` is a place: `sort()`, `swap`, slice assignment | `[D]T`, `List`, `Str`, `mut` slices (R07.3.6) |

**Cost rule.** Views are \(O(1)\) to make and allocate nothing. A
loop over them compiles to the loop you would write by hand. Anything that
builds a container says so: a comprehension, `List(s)`, or a method named
for its result (`sorted`, `tally`, `scan`).

## Slices: which keys survive

Identity everywhere (decision 143; worked through in `04b-keys.md`): **an
index names the same element everywhere it is valid.** A slice narrows the
valid keys and never renames them, for domain arrays, `List` and `Str`
alike, so an index taken from a view is valid on the source.

<!-- spec-only: expression tour with a `...` body -->
```python
let w = a[l..=r]
let best = w.argmax()!                   // a position of a, usable anywhere
for (i, x) in w.items: ...               // i runs l..=r, not 0..
s[2..5][3]                               // s[3]; s[2..5][0] is E0305: 0 isn't a key
w.first!, w.nth(k)                       // first element; k steps from the start
let t = w.rebase                         // O(1) view keyed by a fresh 0..w.len domain t.Idx
```

- **Keys in, keys out.** Library functions answer in the keys they were
  given: `s[l..].search(c)` is a key of `s`, `z_function(s[l..])` is keyed by
  `l..`, `a[l..r].scan(+)` by `a`'s cuts. Lengths and counts stay `Int`. (R07.4.1)
- **Combining two sequences pairs them by walk order**: `==`, `<`,
  `starts_with`, `+`, slice assignment and `zip`. (R07.4.2)
- **`[s.keys]x`** is a table over exactly `s`'s keys; `t.rekey(r)` re-keys
  equal-length cells onto `r` in \(O(1)\). (R07.4.3)
- **Materializing counts from 0:** `.collect()`, `.sorted()`, `List(w)`
  and `Str(w)` build a new sequence keyed from 0. To keep `w`'s keys, build
  a table over them: `[w.keys] k => w[k]` (see R04b.1.6). (R07.4.4)
- **Cuts in walk order.** On any keyed view with cuts (`[D]T`, `List`,
  `Str`, slices, `.rev`), `s.cut_after(i)` and `s.cut_before(i)` are the cuts
  on either side of key `i` in walk order: `i.r` and `i.l` walking forward,
  `i.l` and `i.r` on `.rev`. Both are \(O(1)\) and trap R0101 when `i` is not
  a key of `s`. Generic code over cuts uses them instead of `.l`/`.r`, since
  a view's direction is not part of its type: a scan writes each fold at
  `s.cut_after(i)`. (R07.4.14)
- **When `w[x]` is legal.** One rule covers every keyed view (slices,
  runs, windows, chunks, `rev`, `filter`, cut-keyed scans): a view keeps
  its parent's keys, and `w.nth(k)`, `w.first`, `w.last` are the positional
  accessors on any view (`k` counts walk steps from 0). `w[x]` takes a key:
  - only a view that walks in key order over a range of keys (the
    `Indexed` row above) has `w[x]`; on `rev`, `filter` or a generator it
    is E0305 naming `w.nth(k)`;
  - `x` proven to be in `w.keys` (taken from `w.keys`, `w.items` or a
    search on `w`, or a constant inside constant bounds) reads with no
    check;
  - a literal or constant not proven in `w.keys`, as `r[0]` on a run, is
    E0305: it names `r.nth(0)` for the first element and `a[0]` for the
    parent's key 0;
  - any other `x` is checked: out of `w.keys` traps R0101 naming the key
    and `w.keys`, and never reads the parent outside the view.
  A cut-keyed view (`w.scan(+)`) takes `w`'s cuts the same way. A literal
  that is proven a key still lints (`w[1]` reads key 1, not element 1), as
  does a 0-based counter used as a key (`for k in 0..w.len: w[k]`); both
  suggest `nth`, `items` or `rebase`. A raw `Int` into a domain slice is a
  compile error (decision 388). (R07.4.5)

| View | Indexed by | Walks | Cost |
|---|---|---|---|
| `a[r]` (`r: Range<D>`; `a[j..i]` with cut ends is the piece) | `D`, only inside `r` | `r` in order | \(O(1)\) (R07.4.6) |
| `a.rev` | not indexable: it walks out of key order; `.nth(k)` reads by walk position, and `.items`/`.keys` keep `a`'s keys | backwards | \(O(1)\) (R07.4.7) |
| `a.items` | `D`, gives `(i, a[i])` | in order | \(O(1)\) (R07.4.8) |
| `a.map(f)` | `D`, gives `f(a[i])`, calling `f` on every read | in order | \(O(1)\); `f` per read (R07.4.9) |
| `zip(a, b)` | `a`'s keys, pairing by walk order: the k-th of `a` with the k-th of `b` | in order | \(O(1)\) (R07.4.10) |
| `w.drop(k)`, `w.take(k)`, `w.drop_last(k)`, `w.take_last(k)` | same as `w`, clamped: `w.drop(k)` is `w[w.l.next(k) ?? w.r..]` | in order | \(O(1)\) (R07.4.11) |
| `grid[r]` | the column domain: row `r` of a `[R, C]T` | in order | \(O(1)\) (R07.4.12) |
| `grid[rs, cs]` | `(R, C)` pairs inside `rs` × `cs`; `.items` gives `((x, y), v)` | row-major | \(O(1)\) (R07.4.13) |

`zip` pairs by walk order and is keyed by its first argument; zipping a
`[V]T` with an `[E]T` is still a type error. It stops at the shorter side.
Writing a sequence into a range is slice assignment, not a zip:
`q[start..] = suffix` copies in order, \(O(k)\), and traps in every build if the
lengths differ.

A view borrows its source. Through a `var` source a view is writable and
every in-place verb works on it: `a[l..r].sort()`, `a[l..=r].reverse()`,
`a[r].fill(0)`. Changing the source's length while a view is alive is a
compile error (§128).

## Under the hood

A view is a plain library struct built from a lent source: a reference plus the two
end cuts as Ints. Direction and adapters are in the type, so nothing is
decided at run time and nothing allocates.

<!-- spec-only: lowering sketch: each line's comment is its C++ shape -->
```python
a[l..=r]                  // Slice{src: &a, lo: Int(l.l), hi: Int(r.r)}
a[l..=r].rev.map(f)       // Mapped<Rev<Slice<[V]T>>, F>, F the lambda's own type
```

| Written | Lowers to |
|---|---|
| `a[l..=r]`, `.rev`, `.map(f)` | build the struct, \(O(1)\) (R07.5.1) |
| `w[i]` | debug: check `lo <= i < hi`; then `a[i]` (no offset: indices are kept) (R07.5.2) |
| `w.rev[i]` | error E0305: only a view that walks in key order is indexable; the fix writes `w.rev.nth(k)` (the `k`th in walk order) or `w[i]` (the key) (R07.5.3) |
| `a.map(f)[i]` | `f(a[i])`, inlined, per read (R07.5.4) |
| `for x in w` | `for (i = lo; i < hi; ++i) body(a[i])` (R07.5.5) |
| `for x in w.rev` | the same loop, `hi - 1` down to `lo` (R07.5.6) |
| `w.keys`, `w.len` | `lo..hi`, `hi - lo` (R07.5.7) |
| `List(w)` | the one allocation: a fresh 0-based copy (R07.5.8) |

- **Lending: the rules of `mut a[i]`, over a range.** Any number of read
  views coexist with reads. A `mut` view excludes every other access to what
  it covers; two `mut` views not provably disjoint are a compile error
  (E0601). Halves at one cut (`mut a[..c]` and `mut a[c..]`, or `halve`'s
  two) are provably disjoint. A length change (`push`, `pop`, `clear`) while
  any view or place is alive is a compile error. (R07.5.9)
- **Lifetime: second-class, no annotations.** A value is lent if any part
  of it is (a field, a capture, a generator argument), and a lent value
  lives until its last use in the block that made it. It passes into fns
  (arguments are always lent) and comes back only from a fn declared
  `lends`, whose result borrows every non-plain parameter unless narrowed:
  `fn window(xs: [V]Int, r: Range<V>) -> lends [V]Int = xs[r]`,
  `fn pick(a: [V]Int, b: [V]Int) -> lends(a) [V]Int`. It is a lent value and is kept
  like one (R05.7.8): in a `let`, `var`, field or container that dies no
  later than its sources, never in a global from a fn or in an escaping
  closure. Keeping one longer is `List(w)`. An unnamed temporary lives to the end of its statement, and the temporaries of a
`for` header live for the whole loop: `for r in a.sorted().runs():` is valid. (R07.5.10)
- **Parameters.** A read-only `xs: [V]T` takes an array or any `Indexed`
  over `V` (a `mut` one, any `IndexedMut`); it is a template parameter, so
  each call site gets its own fused loop. `List<T>` and `Str` parameters
  take only that container (chapter 4b, "User helpers"). (R07.5.11)
- **Walk-only views.** `filter`, `take_while` and generators are `Seq`
  only: the `if` or the generator body is inlined into the `for`. (R07.5.12)

## Generators

<!-- given:
  let n = 12
  let lim = 5 -->
```python
fn divisors(n: Int) -> yields Int:
    for d in 1..=isqrt(n):
        if n % d == 0:
            yield d
            if d * d != n: yield n / d

for d in divisors(n):
    if d > lim: break           // break, continue, return, break 'outer, loop else
```

- **Push, not pull.** The generator runs and the `for` body runs at each
  `yield`, so a non-recursive generator inlines to a plain loop. Recursive
  generators (DFS, subset enumeration) work the same way, including `break`
  and `return` out of the body (§126). (R07.6.1, permits)
- **A generator value is a closure you walk instead of call.** `let g =
  divisors(n)` stores the fn and its arguments and runs nothing; each walk
  runs the body from the top, with fresh locals and the arguments as they
  are now. It is owned or lent by what it captured: `divisors(12)` is
  owned, `dfs(graph, from: [s])` is lent, `dfs(g, from: [s], seen: mut seen)` is `mut`-lent. Walking a
  `mut`-lent one twice is legal (the second run sees what the first wrote)
  and linted. Keep the results with `List(g)`. (R07.6.2)
- **Steering.** The yielded item may carry methods the generator reads after
  `yield` returns: `for step in dfs(g, from: [s]): if bad(step.v): step.prune()`.
  This is ordinary library code; any user generator can do it. (R07.6.3)
- **No lockstep.** Two generators can't be walked in step, because each
  wants to own the loop. Materialize one (`List(g)`), or zip indexed
  sequences. `zip`'s first argument is `Indexed` and keys the pairs; each
  later one is `Indexed` or the `.rev` of an `Indexed`, read by walk
  position, so `zip(a, a.rev)` pairs each element with its mirror. (R07.6.4)

## Adapters (each makes a view: \(O(1)\), lazy)

| Form | Result | Note |
|---|---|---|
| `s.map(f)` | `Indexed` if `s` is, else `Seq` | `f` runs on every read (R07.7.1) |
| `s.filter(p)`, `s.take_while(p)`, `s.skip_while(p)` | `Seq` | (R07.7.2) |
| `s.take(k)`, `s.drop(k)` | an \(O(1)\) view on an `Indexed`; a `Seq` otherwise | consuming a view from the front is `w = w.drop(1)` (R07.7.3) |
| `chain(a, b, ...)` | `Seq` of each argument in turn, any number of them, as `min` and `max` take | (R07.7.4) |
| `s.adjacent` | `Seq<(T, T)>`, adjacent pairs | `@get`, \(O(1)\) per step (R07.7.5) |
| `s.pairs()` | `Seq<(T, T)>`, every pair `x` before `y` | \(O(n^2)\) to walk, so a call (R07.7.6) |
| `s.with_prev` | `Seq<(?T, T)>`, starting `(None, first)` | path walks that treat the first step apart (R07.7.7) |
| `s.cyclic_pairs` | `Seq<(T, T)>`, plus (last, first) | polygons, rings (R07.7.8) |
| `product(a, b)` | `Seq<(T, U)>`, every `(x, y)` with `x` from `a` and `y` from `b`, `a` outer | \(O(1)\) per step; `b` is walked once per `x`, so it must be `Indexed` (E0606 on a generator) (R07.7.12) |
| `a.windows(k)`, `a.chunks(k)` | `Seq` of views of `a` | each view keeps `a`'s indices (R07.7.9) |
| `a.runs(by: k)` | `Seq` of views: maximal equal-key runs | `.keys` of each is its range (R07.7.10) |
| `s.step(k)` | every k-th element | ranges and indexed (R07.7.11) |

**Parentheses mean work.** Only \(O(1)\) members read without `()`: `@get`
properties such as `.len`, and views and generators, which cost \(O(1)\) to
make and are paid for as you walk them (`.rev`, `.items`, `.adjacent`, `.keys`,
`g.edges`, `t.tour`). Anything that computes or allocates up front is a
call, however few arguments it takes: `g.transpose()`, `t.centroid()`,
`a.sorted()`, `.collect()`. The spec check rejects an `@get` whose documented
cost isn't \(O(1)\) to make.

## Making containers

<!-- spec-only: expression tour, one form per line -->
```python
[f(x) for x in s if p(x)]      // List<U>; reserves exactly when there's no filter
[V] v => f(v)                  // [V]U, filled by f
s.filter(p).map(f).collect()   // end of a chain: the type the context expects, else List
List(s)   Set(s)   Str(s)      // conversions of anything that isn't a chain: List(0..n)
[1..=n]                       // a one-element list holding a range; the list 1..n is List(1..=n)
```

`.collect()` and the constructors are one operation, \(O(n)\) and allocating;
the postfix form is for chains, the prefix form for single values. So a
`Seq<Char>` becomes a word as `Str(cs)` or `let w: Str = cs.collect()`,
and `sep.join(parts)` is for joining `Str` parts. `List(s)` takes one
sequence and copies it; a list of given values is the literal `[a, b]`,
and a list holding one list is `[xs]` (decision 348). (R07.8.1)

**Lazy chains redo their work on every consumption.** A `map` calls `f` on
every read; a `filter` calls `p` on every walk. Lint flags a lazy chain
walked twice, or built outside a loop and consumed inside it, and a lazy
`map`/`filter` whose closure mutates anything (`rng.int(..)`, `count += 1`),
suggesting `.collect()` or a comprehension. Walking a `mut`-lent value twice
(`let a = dfs(g, from: [s], seen: mut seen)`, then two walks of `a`) gets the sharper message,
since the second walk reruns against what the first wrote.

## Folds

| Form | Result | Cost |
|---|---|---|
| `s.sum()`, `s.prod()` | `T`; empty is the identity | \(O(n)\) (R07.9.1) |
| `s.fold(op)` | `T` when `op` has an identity (`+`, `max` on a bounded type, `gcd`), else `?T` | \(O(n)\) (R07.9.2) |
| `s.fold(f, init: a)` | `Acc`: a left fold into any accumulator, `init` when empty | \(O(n)\) (R07.9.2) |
| `s.min()`, `s.max()`, `s.min(by: k)`, `s.max(ties: Ties.Last)` | `?T`, `None` when empty; first on ties, the last with `ties: Ties.Last` | \(O(n)\) (R07.9.3) |
| `s.min_k(k, by: f)`, `s.max_k(k, by: f)` | `List<T>`, the `k` smallest or largest (every element when `k >= n`) in no set order; earlier wins ties | \(O(n)\) expected (R07.9.3) |
| `s.count(p)`, `s.any(p)`, `s.all(p)`; `s.count()` counts every element, so on a `Seq<Bool>` it is an error naming `count(x => x)` (the trues) and `len` (decision 378) | `Int`, `Bool` | \(O(n)\) (R07.9.4) |
| `s.find(p)`, `s.rfind(p)` | `?T`, linear scan (binary search is `r.first_true`) | \(O(n)\) (R07.9.5) |

**A fold costs what the receiver's type says.** The costs above are the
generic walks every `Seq` gets. They are default methods of the
capability trait that provides them (`Seq`, `Sized`, `Bidir`,
`Indexed`), so a type with a faster way overrides the method in its
`impl`, written in CP Lang with its own documented cost. A range's `impl`
gives `sum()`, `min()`, `max()` and `count()` as closed forms, so
`(3..m).step(2).sum()` is \(O(1)\). In the same way `Prefix.fold` is \(O(1)\)
and `Bitset.len` is a popcount. Ordinary method resolution picks the
method (R08.1.4), generic code included, and the compiler never pattern
matches a pipeline. User types get the same power. Parentheses still
mean "may do work" (decision 140), and the cost is the one the resolved
method documents; hover and `cpl explain` name that method and its cost
(decision 486). (R07.9.6)

Position queries are folds over `a.keys`, because the keys are a range of
positions. `argmin`/`argmax` are kept as the named form of the first
line below, with the same `ties:` option (first by default); there are no
`first_at` or `order` twins:

<!-- spec-only: expression tour, one form per line -->
```python
a.keys.min(by: i => a[i])!         // argmin, first on ties
a.keys.find(i => a[i] > x)         // first position with a[i] > x, ?D
a.keys.sorted(by: i => a[i])       // argsort: List<D>
a[l..r].keys.max(by: i => a[i])!   // argmax inside a piece, still a position of a
```

## Scans are prefix sums over cuts

`a.scan(op)` gives a `Prefix<D, T>`: n + 1 exclusive folds indexed by `D`'s
cuts (`xs.scan(+).cut(k)` folds the first `k`; `xs.scan(+)[k]` with an `Int` `k` is E0320). Walking backwards gives suffix folds, indexed by the same cuts:

<!-- spec-only: expression tour, one form per line -->
```python
let pre = a.scan(+)        // pre[c]: fold of everything left of cut c
let suf = a.rev.scan(+)    // suf[c]: fold of everything right of cut c
pre[r.r] - pre[r.l]        // any range r
pre[v]                     // a position: the fold through v, same as pre[v.r]
Prefix(a).fold(r)          // same, as the `Prefix` struct (`+` only)
pre[v.l] + a[v] + suf[v.r] // the whole array, split at position v
```

A `Prefix` takes both kinds of index and the types keep them apart: a cut
folds what lies left of it, a position folds through itself. The inclusive
query (`pref_min[v]`) is the common one and needs no `.r`.

\(O(n)\) time and memory. This replaces `scan`, `suffix_scan`, `excl_fold`,
`scan_in_place` and `Prefix(...)` construction.

## Ordering and grouping

| Form | Meaning | Cost |
|---|---|---|
| `s.sort(by: k)`, `s.reverse()`, `s.dedup()`, `s.fill(x)` | in place: needs a `var` source; works on views | \(O(n \log n)\), \(O(n)\), \(O(n)\), \(O(n)\) (R07.11.1) |
| `s.sorted(by: k)`, `s.deduped()` | new `List` | same, allocates (R07.11.2) |
| `s.group(by: k)` | `Map<K, List<T>>`, each list in walk order | \(O(n)\) expected (R07.11.3) |
| `s.tally(by: k)` | `Counter<K>` | \(O(n)\) expected (R07.11.4) |
| `compress(s)` | `(domain C, comp)`: values sorted and deduped, as a fresh domain | \(O(n \log n)\) (R07.11.5) |

Sorts are stable. Descending is a key, not a flag: `by: x => desc(x.score)`;
`sort` and `sorted` take no `desc:` argument (decision 366). `sorted()`
returns a fresh `List` keyed from 0, even from a slice that keeps its
source's keys.
Every in-place verb that has a result-named twin follows the same pattern
(`sort`/`sorted`, `dedup`/`deduped`); `.rev` (view) and `reverse()` (in
place) are different jobs, not twins.

## More vocabulary

| Form | Meaning | Cost |
|---|---|---|
| `s.drop_while(p)` | skip the leading elements where `p` holds, yield the rest | \(O(1)\) per step (R07.12.1) |
| `s.flat_map(f)`, `s.flatten()` | each element to a sequence, all inner elements in order | \(O(1)\) per element (R07.12.2) |
| `iterate(x, step)` | `x, step(x), step(step(x)), ...` until `step` gives `None` | \(O(1)\) per step (R07.12.3) |
| `s.is_sorted(by:)`, `s.is_subsequence(t)` | the two common checks | \(O(n)\) (R07.12.4) |
| `a.partition_point(ok)`, `a.equal_range(x, by:)` | on sorted `a`: the cut where monotone `ok` turns true (an `Int` on a `List`); the range equal to `x` | \(O(\log n)\) (R07.12.5) |
| `a.next_permutation()`, `a.prev_permutation()` | step to the neighbouring arrangement in place; `false` past the end | amortized \(O(1)\) (R07.12.6) |
| `a.rotate_left(k)`, `a.rotate_right(k)`; `s.rotated_left(k)`, `s.rotated_right(k)` | rotate in place; a rotated copy | \(O(n)\) (R07.12.7) |
| `m.get_or_insert(k, v)` | `m[k]`, storing `v` first if absent: `ids.get_or_insert(name, ids.len)` | \(O(\log n)\) (R07.12.8) |
| `m[k]`, `m[k] = v`, `m[k] op= v` on a `Map` | a read traps when `k` is absent (`m.get(k)` is the `?V`); `=` inserts or replaces; `op=` starts an absent key at the default of `V` (`0`, `""`, empty) | \(O(\log n)\), \(O(1)\) expected for a hash map (R07.12.9) |

## Answers to round 2's questions

1. **In-place vs copy:** both, by rule: a verb mutates, its result-named
   twin allocates.
2. **Folds on possibly-empty input:** `NonEmpty` is gone. Identity-backed
   folds (`sum`, `fold(+)`) are plain `T`; `min`/`max` are `?T` and you
   write `a.max()!` when you know.
3. **`scan`:** n + 1 values indexed by cuts; suffix folds come from `.rev`.

## Open

None. Indexed sequences use `.keys` and domains `V.range`; `.collect()`
keeps its name (decision 186).

## What goes away

`View<T>`, `view()`, 0-based `.at(i)` / `.try_at(i)` on `Indexed`,
`drop_first` (now `drop`), `skip`, `step_by`, `first_at`,
`order`, `chunk_by`, `run_ranges`, `excl_fold`, `suffix_scan`,
`scan_in_place`. Each is a view, a fold over `.keys`, or `.rev.scan`.

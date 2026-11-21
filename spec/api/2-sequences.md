# Round 2: sequences and views

Built on round 1: arrays are `[D]T`, ranges are runs of a domain with cut
ends, `.first`/`.last` are optional.

## Key decisions

1. **Generators are the only iterator.** `-> yields T`, no `next()`; values
   are pushed into the loop body, so they compile to the hand-written loop.
   `break`/`return` work through them; lockstep is `List(g)`.
2. **Two capabilities.** `Seq` walks; `Indexed` adds O(1) `s[i]`, `.len`,
   `.keys`. Adapters keep `Indexed` when they can stay O(1) (slices, `.rev`,
   `map`, `take`/`drop`, `zip`); `filter`, `take_while` and generators are
   `Seq`.
3. **Laziness is explicit.** A chain reruns on every walk. Allocation happens
   only at `.collect()`, a comprehension or `List(...)`. Lint flags a chain
   walked twice and side-effecting closures.
4. **Identity vs offset.** A domain slice keeps its positions
   (`a[l..=r][i] == a[i]`); a `List`/`Str` slice counts from 0.
5. **Views are borrows with the rules of `mut a[i]`.** No lifetime
   annotations, never stored, returned only through `lends`. In-place verbs
   work on views (`a[l..=r].sort()`); each has a copying twin (`sorted`).
6. **Prefix scans are indexed by cuts.** `pre[c]` folds left of `c`;
   `pre[v]` folds through `v`.
7. **Costs show.** O(1) things are getters or views, allocation is named,
   options are named arguments (`max(by:)`), and there is no overloading.

## The model in one paragraph

There is one protocol and one capability on top of it. A **`Seq<T>`** is
anything a `for` can walk: it has `iter() -> yields T`. A **generator**
(`fn f(...) -> yields T`) is the only iterator; there is no `next()`. An
**`Indexed<D, T>`** is a `Seq` with O(1) `s[i]` over a domain `D` and a
known index range `s.keys: Range<D>`. Every array, list, string, range and
view is `Indexed`. A **view** is a source plus a range and a direction. A
domain position is an identity, so a slice of a `[V]T` keeps `V`'s
positions; an `Int` index into a `List` or `Str` is an offset, so a slice
of one counts from 0.

```python
a[l..=r]          # view of a over l..=r; a[l..=r][i] is a[i], i must be in l..=r
s[2..5]           # a List/Str slice counts from 0: s[2..5][0] is s[2]
a[l..=r].keys     # l..=r
a.rev             # walks a backwards; a.rev[i] is still a[i]
a.map(f)          # Indexed<D, U>: (a.map(f))[i] is f(a[i]), O(1) per read
a.items           # (i, a[i]) pairs with a's own indices
```

## Kinds

| Kind | What it guarantees | Examples |
|---|---|---|
| `Seq<T>` | `for x in s` walks it in order; each `for` runs it from the start | generators, `s.filter(p)`, `chain(a, b)`, `g.out(u)` |
| `Indexed<D, T>` | `Seq` + `s[i]` for `i: D` in O(1), `s.keys: Range<D>`, `s.len` | `[D]T`, `List<T>` and `Str` (`D = Int`), ranges, slices, `.rev`, `.map(f)`, `zip(a, b)` |

**Cost rule.** Views and adapters are O(1) to make and allocate nothing. A
loop over them compiles to the loop you would write by hand. Anything that
builds a container says so: a comprehension, `List(s)`, or a method named
for its result (`sorted`, `tally`, `scan`).

## Domain views keep their positions; offset views count from 0

The key type decides. A domain position names an element (round 1's index
safety), so it means the same element in the array and in every view of it:
an index taken from a view is valid on the source, and nothing is rebased.

```python
let w = a[l..=r]
let best = w.keys.max(by: i => a[i])!    # a position of a, usable anywhere
for (i, x) in w.items: ...               # i runs l..=r, not 0..
w[V(0)]                                  # traps unless V(0) is in l..=r
```

`List<T>` and `Str` are indexed by `Int` offsets, and an offset has no
identity to keep, so their slices count from 0, as in Python:
`s[2..5][0]` is `s[2]`, still an O(1) view. Keeping offsets would make
`rest[i]` for `i in 0..rest.len` silently read the wrong element whenever
`i` lands inside the slice, and no type stops a plain `Int`. A domain slice
can't have that bug: `w[i]` takes a `V`, and a `V` outside the slice traps.

An integer literal is never a position into a domain slice: `blk[1..]` on
`blk = a[l..r]` is an error ("slices keep their positions"), suggesting
`blk.drop(1)`. On a whole array the literal reads as round 1 says.

| View | Indexed by | Walks | Cost |
|---|---|---|---|
| `a[r]` (`r: Range<D>`; `a[j..i]` with cut ends is the piece) | `D`, only inside `r` | `r` in order | O(1) |
| `a.rev` | `D` | backwards | O(1) |
| `a.items` | `D`, gives `(i, a[i])` | in order | O(1) |
| `a.map(f)` | `D`, gives `f(a[i])`, calling `f` on every read | in order | O(1); `f` per read |
| `zip(a, b)` | the shared `D`, gives `(a[i], b[i])` over `a.keys.clip(b.keys)` | in order | O(1) |
| `w.drop(k)`, `w.take(k)`, `w.drop_last(k)`, `w.take_last(k)` | same as `w`, clamped: `w.drop(k)` is `w[w.l.next(k) ?? w.r..]` | in order | O(1) |
| `grid[r]` | the column domain: row `r` of a `[R, C]T` | in order | O(1) |

`zip` needs both sides indexed by the same domain, so zipping a `[V]T` with
an `[E]T` is a type error. Two lists zip over the shorter one's indices.
Writing a sequence into a range is slice assignment, not a zip:
`q[start..] = suffix` copies in order, O(k), and traps in every build if the
lengths differ.

A view borrows its source. Through a `var` source a view is writable and
every in-place verb works on it: `a[l..r].sort()`, `a[l..=r].reverse()`,
`a[r].fill(0)`. Changing the source's length while a view is alive is a
compile error (§128).

## Under the hood

A view is a library struct that lends its source: a reference plus the two
end cuts as Ints. Direction and adapters are in the type, so nothing is
decided at run time and nothing allocates.

```python
a[l..=r]                  # Slice{src: &a, lo: Int(l.l), hi: Int(r.r)}
a[l..=r].rev.map(f)       # Map<Rev<Slice<[V]T>>, F>, F the lambda's own type
```

| Written | Lowers to |
|---|---|
| `a[l..=r]`, `.rev`, `.map(f)` | build the struct, O(1) |
| `w[i]` | debug: check `lo <= i < hi`; then `a[i]` (no offset: indices are kept) |
| `w.rev[i]` | `a[i]`; only the walk order differs |
| `a.map(f)[i]` | `f(a[i])`, inlined, per read |
| `for x in w` | `for (i = lo; i < hi; ++i) body(a[i])` |
| `for x in w.rev` | the same loop, `hi - 1` down to `lo` |
| `w.keys`, `w.len` | `lo..hi`, `hi - lo` |
| `List(w)` | the one allocation: a fresh 0-based copy |

- **Lending: the rules of `mut a[i]`, over a range.** Any number of read
  views coexist with reads. A `mut` view excludes every other access to what
  it covers; two `mut` views not provably disjoint are a compile error
  (E0602). Halves at one cut (`mut a[..c]` and `mut a[c..]`, or `halve`'s
  two) are provably disjoint. A length change (`push`, `pop`, `clear`) while
  any view or place is alive is a compile error.
- **Lifetime: second-class, no annotations.** A view lives at most as long
  as its source's scope. It passes into fns (arguments are always lent) and
  comes back only from a fn that `lends` it from a parameter:
  `fn window(xs: [V]Int, r: Range<V>) lends [V]Int = xs[r]`. It never goes
  in a field, container, global or escaping closure. Keeping one is
  `List(w)`.
- **Parameters.** A read-only `xs: [V]T` takes an array or any `Indexed`
  over `V`; it is a template parameter, so each call site gets its own fused
  loop.
- **Filtered sources.** `filter`, `take_while` and generators are `Seq`
  only: the `if` or the generator body is inlined into the `for`.

## Generators

```python
fn divisors(n: Int) -> yields Int:
    for d in 1..=n.isqrt():
        if n % d == 0:
            yield d
            if d * d != n: yield n / d

for d in divisors(n):
    if d > lim: break           # break, continue, return, break 'outer, loop else
```

- **Push, not pull.** The generator runs and the `for` body runs at each
  `yield`, so a non-recursive generator inlines to a plain loop. Recursive
  generators (DFS, subset enumeration) work the same way, including `break`
  and `return` out of the body (§126).
- **A generator value is a recipe.** `let g = divisors(n)` stores the call;
  each `for` over `g` runs it again. Keep the results with `List(g)`.
- **Steering.** The yielded item may carry methods the generator reads after
  `yield` returns: `for step in dfs(g, s): if bad(step.v): step.prune()`.
  This is ordinary library code; any user generator can do it.
- **No lockstep.** Two generators can't be walked in step, because each
  wants to own the loop. Materialize one (`List(g)`), or zip indexed
  sequences. `zip` takes only `Indexed`.

## Adapters (O(1) to make, lazy)

| Form | Result | Note |
|---|---|---|
| `s.map(f)` | `Indexed` if `s` is, else `Seq` | `f` runs on every read |
| `s.filter(p)`, `s.take_while(p)`, `s.skip_while(p)` | `Seq` | |
| `s.take(k)`, `s.drop(k)` | an O(1) view on an `Indexed`; a `Seq` otherwise | consuming a view from the front is `w = w.drop(1)` |
| `chain(a, b)` | `Seq` | |
| `s.pairs` | `Seq<(T, T)>`, adjacent pairs | `@get`, O(1) per step |
| `s.with_prev` | `Seq<(?T, T)>`, starting `(none, first)` | path walks that treat the first step apart |
| `s.cyclic_pairs` | `Seq<(T, T)>`, plus (last, first) | polygons, rings |
| `a.windows(k)`, `a.chunks(k)` | `Seq` of views of `a` | each view keeps `a`'s indices |
| `a.runs(by: k)` | `Seq` of views: maximal equal-key runs | `.keys` of each is its range |
| `s.step(k)` | every k-th element | ranges and indexed |

Every O(1) view and adapter that takes no arguments is an `@get` property
(`.rev`, `.items`, `.pairs`, `.keys`); everything with arguments is a call.

## Making containers

```python
[f(x) for x in s if p(x)]      # List<U>; reserves exactly when there's no filter
[V] v => f(v)                  # [V]U, filled by f
s.filter(p).map(f).collect()   # end of a chain: the type the context expects, else List
List(s)   Set(s)   Str(s)      # conversions of anything that isn't a chain: List(0..n)
```

`.collect()` and the constructors are one operation, O(n) and allocating;
the postfix form is for chains, the prefix form for single values.

**Lazy chains redo their work on every consumption.** A `map` calls `f` on
every read; a `filter` calls `p` on every walk. Lint flags a lazy chain
walked twice, or built outside a loop and consumed inside it, and a lazy
`map`/`filter` whose closure mutates anything (`rng.int(..)`, `count += 1`),
suggesting `.collect()` or a comprehension.

## Folds

| Form | Result | Cost |
|---|---|---|
| `s.sum()`, `s.prod()` | `T`; empty is the identity | O(n) |
| `s.fold(op)` | `T` when `op` has an identity (`+`, `max` on a bounded type, `gcd`), else `?T` | O(n) |
| `s.min()`, `s.max()`, `s.min(by: k)` | `?T`, `none` when empty; first on ties | O(n) |
| `s.count(p)`, `s.any(p)`, `s.all(p)` | `Int`, `Bool` | O(n) |
| `s.first(p)`, `s.last(p)` | `?T`, linear scan (binary search is `r.first_true`) | O(n) |

Position queries are folds over `a.keys`, because the keys are a range of
positions. No `argmin`, `first_at` or `order` twins:

```python
a.keys.min(by: i => a[i])!         # argmin, first on ties
a.keys.first(i => a[i] > x)        # first position with a[i] > x, ?D
a.keys.sorted(by: i => a[i])       # argsort: List<D>
a[l..r].keys.max(by: i => a[i])!   # argmax inside a piece, still a position of a
```

## Scans are prefix sums over cuts

`a.scan(op)` gives a `Prefix<D, T>`: n + 1 exclusive folds indexed by `D`'s
cuts. Walking backwards gives suffix folds, indexed by the same cuts:

```python
let pre = a.scan(+)        # pre[c]: fold of everything left of cut c
let suf = a.rev.scan(+)    # suf[c]: fold of everything right of cut c
pre[r.r] - pre[r.l]        # any range r
pre[v]                     # a position: the fold through v, same as pre[v.r]
pre.fold(r)                # same, for any op with an inverse; matches seg.fold(r)
pre[v.l] + a[v] + suf[v.r] # the whole array, split at position v
```

A `Prefix` takes both kinds of index and the types keep them apart: a cut
folds what lies left of it, a position folds through itself. The inclusive
query (`pref_min[v]`) is the common one and needs no `.r`.

O(n) time and memory. This replaces `scan`, `suffix_scan`, `excl_fold`,
`scan_in_place` and `Prefix(...)` construction.

## Ordering and grouping

| Form | Meaning | Cost |
|---|---|---|
| `s.sort(by: k)`, `s.reverse()`, `s.dedup()`, `s.fill(x)` | in place: needs a `var` source; works on views | O(n log n), O(n), O(n), O(n) |
| `s.sorted(by: k)`, `s.deduped()` | new `List` | same, allocates |
| `s.group(by: k)` | `Map<K, List<T>>` | O(n) expected |
| `s.tally(by: k)` | `Counter<K>` | O(n) expected |
| `compress(s)` | `(domain C, comp)`: values sorted and deduped, as a fresh domain | O(n log n) |

Sorts are stable. Descending is a key, not a flag: `by: x => desc(x.score)`.
Every in-place verb that has a result-named twin follows the same pattern
(`sort`/`sorted`, `dedup`/`deduped`); `.rev` (view) and `reverse()` (in
place) are different jobs, not twins.

## Answers to round 2's questions

1. **In-place vs copy:** both, by rule: a verb mutates, its result-named
   twin allocates.
2. **Folds on possibly-empty input:** `NonEmpty` is gone. Identity-backed
   folds (`sum`, `fold(+)`) are plain `T`; `min`/`max` are `?T` and you
   write `a.max()!` when you know.
3. **`scan`:** n + 1 values indexed by cuts; suffix folds come from `.rev`.

## Open

- **`V.all` and `a.all` rename.** `a.all` (the index range) collides with
  `s.all(p)` (the predicate), and there's no overloading (§127). Proposed:
  `a.keys` for every indexed sequence (as `Map.keys`), `V.range` for a
  domain. Round 1 is updated to match.
- **`.collect()` naming.** Rust's word; `.list()` is the alternative if
  the target is almost always a `List`.

## What goes away

`View<T>`, `view()`, 0-based `.at(i)` / `.try_at(i)` on `Indexed`,
`drop_first` (now `drop`), `skip`, `step_by`, `argmin`/`argmax`, `first_at`,
`order`, `chunk_by`, `run_ranges`, `excl_fold`, `suffix_scan`,
`scan_in_place`. Each is a view, a fold over `.keys`, or `.rev.scan`.

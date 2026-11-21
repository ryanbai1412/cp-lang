# Round 2: sequences and views

Built on round 1: arrays are `[D]T`, ranges are runs of positions with cut
ends, `NonEmpty` decides plain vs optional.

## The model

Two kinds of sequence, and one rule for costs.

| Kind | What it is | Examples |
|---|---|---|
| `Seq<T>` | anything a `for` can walk, once, in order | generators, `xs.map(f)`, `zip(a, b)`, `g.out(u)` |
| `Indexed<D, T>` | a `Seq` with O(1) `s[i]` over a domain `D` | `[D]T`, `List<T>`, `Str`, ranges, views |

**Cost rule.** Adapters and views are O(1) to make and allocate nothing.
Anything that builds a container says so: a comprehension, `List(g)`, or a
method named for its result (`sorted`, `tally`). A loop over an adapter
compiles to the loop you would write by hand.

## Views (O(1), no copy, indexed)

```python
a[l..=r]   a[j..i]   a[..k]        # slices keep a's domain: a[2..5][3] is a[3]
a.rev                              # reversed view
a.items                            # (index, value) pairs
grid[r]                            # row view of a [R, C]T
```

A view borrows its source. Writing through `mut a[l..=r]` is allowed;
changing `a`'s length while a view is alive is a compile error (§128).

## Adapters (O(1) to make, lazy, single pass)

```python
xs.map(f)   xs.filter(p)   xs.take(k)   xs.skip(k)   xs.take_while(p)
zip(a, b)   chain(a, b)    xs.pairs (adjacent)   xs.windows(k)   xs.chunks(k)
xs.scan(op)          # running folds, see below
```

## Making containers

```python
[f(x) for x in xs if p(x)]         # List; reserves when there is no filter
[V] v => f(v)                      # array over a domain, filled by f
List(g)   Set(g)   Map(g)          # collect any Seq
```

## Folds

| Form | Result | Cost |
|---|---|---|
| `xs.sum()`, `xs.prod()` | `T` (empty is the identity) | O(n) |
| `xs.fold(op)` | `T` if the source is `NonEmpty` or `op` has an identity, else `?T` | O(n) |
| `xs.min()`, `xs.max()`, `xs.min(by: k)` | same rule as `fold` | O(n) |
| `a.argmin(by: k)`, `a.argmax` | the position, `D` or `?D` by the same rule | O(n) |
| `xs.count(p)`, `xs.any(p)`, `xs.all(p)` | `Int`, `Bool` | O(n) |
| `xs.first(p)`, `xs.last(p)` | `?T`; `a.first_at(p)` gives the position | O(n) |

`op` is any operation value: `xs.fold(gcd)`, `xs.fold(max)`, `xs.fold(+)`.

## Scans are prefix sums

`a.scan(op)` over `[D]T` is a `[D.bounds]T` of n + 1 exclusive folds, so a
range is two cut reads and nothing is off by one:

```python
let pre = a.scan(+)                # pre[c]: fold of everything left of cut c
pre[r.r] - pre[r.l]                # any range r
pre.fold(r)                        # same, for any invertible op
let suf = a.rev.scan(+)            # suffix folds, indexed by a.rev's cuts
```

This replaces `scan`, `suffix_scan`, `excl_fold` and `Prefix`.

## Ordering and grouping

| Form | Meaning | Cost |
|---|---|---|
| `xs.sort(by: k)`, `xs.reverse()`, `xs.dedup()` | in place, needs a `var` | O(n log n), O(n), O(n) |
| `xs.sorted(by: k)`, `xs.deduped()` | new List | same, allocates |
| `a.order(by: k)` | `List<D>`: positions sorted by key (argsort) | O(n log n) |
| `a.runs(by: k)` | `Seq<Range<D>>` of maximal equal-key runs | O(n) |
| `xs.group(by: k)` | `Map<K, List<T>>` | O(n) |
| `xs.tally(by: k)` | `Counter<K>` | O(n) |
| `compress(xs)` | `(domain C, comp)`: values sorted, deduped, as a domain | O(n log n) |

Descending order is a key, not a flag: `by: x => desc(x.score)`.

## Questions (my recommendation first)

1. **In-place vs copy.** Every mutating verb has a result-named twin
   (`sort`/`sorted`, `dedup`/`deduped`) as a rule, not a coincidence.
   Alternative: in-place only; a copy is `var ys = xs.clone(); ys.sort()`.
2. **Folds on possibly-empty input** reuse round 1's `NonEmpty`: `a.max()` is
   `T` over `[V]T` when `V: NonEmpty`, else `?T`. Alternative: always `?T`.
3. **`scan` returns the n + 1 cut-indexed prefix**, the one spelling for
   prefix sums. Alternative: inclusive `[D]T` like C++ `partial_sum`.

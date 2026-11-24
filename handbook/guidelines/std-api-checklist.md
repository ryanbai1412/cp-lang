# std API checklist

| | |
|---|---|
| Kind | Checklist |
| Owner | stdlib |
| Describes | every public signature in `std/*.cp` and `std/prelude.cp`; the reasoning is in [std-api.md](std-api.md) |

Every rule a public std signature must meet, one per heading. Each has a
stable id (`std-api/R4`), one sentence, its reason, an example and what
enforces it. Lints, tests and the [stdlib quality register](../domains/stdlib.md)
cite these ids instead of restating the rule. For a case not covered here,
decide it from [std-api.md](std-api.md), then add the rule.

# Reference

## Names

### R1. Every name comes from the lexicon

One idea has one name in every module; a new idea adds a row here in the commit that first uses it.

- **Why:** a contestant guesses the next type's calls from the last one.
- **Example:** every set-like container uses `add` and `remove`, never `insert` and `erase`.
- **Enforced by:** `scripts/std-api-lint.py` fails a public std fn named by a word in the right-hand column that no row's middle column uses; `scripts/spec-gate.py vocab` bans phrasings across std, spec and docs.

| Idea | Name | Not |
|---|---|---|
| size | `s.len` (a getter) | `len()`, `size()`, `count()` |
| empty test | `s.is_empty` | `empty()` |
| the empty value of a type | `T.empty` | |
| insert into a set-like container | `s.add(x)`, returns `Bool` (was new) | `insert`, `put` |
| delete from a container | `s.remove(x)`, returns `Bool` (maps: the removed `?V`) | `erase`, `delete`, `discard` |
| flip membership | `s.toggle(x)` | `flip` |
| negation | `-x`, `not b`, `l.neg` (a literal) | `flip` |
| membership | `x in s`, method form `s.contains(x)` | `has`, `count(x) > 0` |
| back of a sequence | `push`, `pop` | `append`, `push_back` |
| insert at a position of a list | `a.insert(i, x)` | |
| splice a whole structure onto an end | `t.splice_back(other)`, `t.splice_front(other)` | `append` |
| front of a sequence | `push_front`, `pop_front` | `appendleft`, `shift` |
| ends, possibly absent | `first`, `last` (`?T`) | `front`, `back` |
| the reverse of a direction or a matching | `d.back` (the opposite direction), `m.back(r)` (`r`'s partner) | |
| size of the component or subtree holding `v` | `d.size(v)` | |
| move a function's argument (math) | `p.shift(c)` (`p(x + c)`), `f.shift(d)` | |
| monotone search | `r.first_true(ok)`, `r.last_true(ok)`; the cut: `r.bsearch(ok)`, `a.bsearch(ok)` | `binary_search`, `partition_point` |
| point read and write | `t.get(i)`, `t.set(i, x)`, `t.update(i, f)` | `query`, `modify` |
| range fold | `t.fold(r)`; everything is `t.all` | `query`, `sum(l, r)` |
| lazy range action | `t.apply(r, f)` | `update(l, r, x)` |
| boundary search on a structure | `t.max_right(from, ok)`, `t.min_left(to, ok)` | |
| copy that sorts or reverses | `a.sorted()`, `a.rev` | `sorted(a)` |
| move contents out | `s.take()` | `swap with empty` |
| count values per key | `xs.tally()`, a `Counter` | `Counter(xs)` |
| library-domain size, ordinal, value | `T.count`, `x.ord`, `T.nth(k)` | `K`, `to_idx`, `from_idx` |

Sorted lookups on containers (`first_ge`, `last_le`, cursors) and what a
search returns are an open design question for Ryan; until it is settled,
follow the existing type's spelling and add nothing new.

### R2. Textbook algorithms keep the editorial's name

An algorithm is named as editorials name it, and a graph or tree algorithm is a free fn taking the graph first.

- **Why:** the contestant searches for the name they read in the editorial.
- **Example:** `dijkstra(g, from: s)`, `scc(g)`, `z_function(s)`, `convex_hull(ps)`.
- **Enforced by:** review.

### R3. A fallible twin is `try_<verb>`

When an operation traps on bad input, its optional form is `try_<verb>` returning `?T`, unless the established name already is the optional form (`get`, `pop`, `first`, `find`).

- **Why:** the short name stays for the common case and the safe form is guessable.
- **Example:** `x.inv` traps on zero, `x.try_inv()` is `?Mod<P>`; `m[k]` traps, `m.get(k)` is `?V`.
- **Enforced by:** `missing` (API-consistency lint).

## Results

### R4. "Nothing" is `?T`, never a sentinel

A search, min, max, `first`, `last` or `pop` that can find nothing returns `?T`; no fn returns `-1`, `len`, `INF` or `0` to mean "none".

- **Why:** a sentinel type-checks and silently corrupts the answer.
- **Example:** `[].max()` is `None`; a `Counter` reading 0 for an unseen key is not a sentinel, since 0 is its true count. Nor is `PalindromicTree.pal_len` being `-1` at the odd root: the length arithmetic uses it, and its doc says so with "not a sentinel".
- **Enforced by:** `scripts/std-api-lint.py` (a numeric result whose doc gives `-1`, `INF` or `len` for nothing; shrink-only, in `scripts/check.sh`).

### R5. A fold of a monoid is `T`; without an identity it is `?T`

An operation with a known identity (`+`, `*`, `min`, `max`, `gcd`, `^`, `|`, `&`, or a `Monoid`) folds to `T`; any other operation still works everywhere and folds to `?T`, lifted by one helper whose identity is `None`.

- **Why:** the contestant shouldn't unwrap a sum, and shouldn't need an identity to use `Window` or a segment tree with a custom op.
- **Example:** `[3, 1, 4].scan(+).fold(1..=2) == 5`; `Monoid<?Int>.of(min).id == None`.
- **Enforced by:** `std/algebra.cp` `Monoid.of` examples in `cpl-conform docs`; `missing` for `Window.all` and the `?T` lifting helper.

## Arguments

### R6. Options are named arguments with defaults

An option is a named argument with a default, never a `_by`, `_key` or `_with` variant of the fn.

- **Why:** one fn to learn; adding an option is a local edit.
- **Example:** `a.sort(by: r => -r.score)`, `bfs(g, from: s)`, `dijkstra(g, from: s, step: max)`.
- **Enforced by:** `scripts/std-api-lint.py` (`f_by`, `f_key`, `f_with` beside `f`; shrink-only, in `scripts/check.sh`); `scripts/spec-gate.py sigs` (spec named arguments std lacks).

### R7. `by:` takes one element and returns its key

`by:` is always a key function of one element; a two-argument comparator is a different option with a different name.

- **Why:** C++ and Python disagree here, and mixing the two gives type errors far from the mistake.
- **Example:** `rows.sort(by: r => (-r.score, r.name))`.
- **Enforced by:** review; the checker's arity error names `by:` (`errors/R2`).

### R8. Required arguments come in reading order

Required arguments are positional: the container first, then what is looked up, then options by name.

- **Why:** neighbouring fns with swapped orders are a guaranteed slip.
- **Example:** `a.count(x)`, `t.fold(l..r)`, `t.max_right(from, ok)`.
- **Enforced by:** `scripts/std-api-lint.py` fails two public fns of one name whose shared required parameters come in another order.

## Keys and indices

### R9. Over a `[D]T`, keys keep their index space

A fn over a `[D]T` takes and returns `D`, `Cut<D>` or `Range<D>`, never a bare `Int`; a view keeps its source's keys and says so in `order:`.

- **Why:** the domain is what lets the checker catch an off-by-one or a wrong index space.
- **Example:** `a.windows(k)` yields slices keyed like `a`; `a.scan(+)` is keyed by `D.cuts`.
- **Enforced by:** `scripts/std-api-lint.py` fails a public fn over a `[D]T` taking a position (`i`, `at`, `l`, `r`, ...) as a bare `Int`.

### R10. A fn that makes a new index space returns it

When a result has its own index space, the fn returns a fresh domain for it.

- **Why:** indices into the result can't be mixed with indices into the input.
- **Example:** `scc(g)` returns a fresh `domain C` of components; `compress(xs)` returns its rank domain.
- **Enforced by:** review.

## Shape

### R11. One form per operation

A fold, sort or query of one value is a method; math on two or more values and generators are free fns; an operation never has both forms by accident.

- **Why:** two spellings double what a reader must recognise.
- **Example:** `xs.sum()`, `xs.sorted(by: k)`; `min(a, b)`, `gcd(a, b)`, `zip(a, b)`.
- **Enforced by:** lint `L0509` (`free_fold`) rewrites the free form of folds; `scripts/std-api-lint.py` fails an operation that is both a free fn and a method of its first parameter's type.

### R12. Every container meets the common traits

Every container claims `Show`, `Eq` and `Seq` and has `len` and `is_empty`, unless its doc says why not.

- **Why:** printing, comparing and walking should never depend on which container you picked.
- **Example:** `print(bag)`, `bag == other`, `for x in bag:` all work on `SortedBag`.
- **Enforced by:** `scripts/spec-gate.py traits` for claimed traits; `scripts/std-trait-matrix.py` (in `scripts/check.sh`, shrink-only) lists every container missing `Show`, a walk or `is_empty`. A struct opts out with `not a container: <why>` in its doc.

### R13. Each type's everyday tasks are one call each, with a case

Every container and range structure lists its journeys below, each one short call with a tested example; a journey that needs a loop, two calls or a sentinel check is an API bug.

- **Why:** this is where contestants feel the library, mid-contest.
- **Example:** removing one copy from a `SortedBag` is `bag.remove(x)`, not a find then an erase.
- **Enforced by:** `scripts/std-api-lint.py` (every journey below has a `// journey:` line in `spec/conformance/std-journeys/<type>.cp`, which the `std-journeys` conformance suite runs).

| Type | Journeys |
|---|---|
| `SortedBag<T>` | add, remove one copy, remove at a cursor, predecessor, successor, count in a range, k-th smallest, walk a range in order |
| `SortedSet<T>` | add returning whether it was new, remove one copy, predecessor, successor, count in a range, k-th smallest, walk a range in order (a set's key is its own cursor) |
| `HashMap<K, V>` / `Counter<K, V>` | count occurrences (`tally`), read with a default, add to a value, walk in a fixed order |
| `SegTree` / `LazySegTree` | point set, point update, range fold, range apply, first index where a prefix holds (`max_right`) |
| `Fenwick` | add at a point, prefix sum, range sum, first prefix reaching a bound |
| `Dsu<V>` | union, same set, set size, list the sets |
| `Window<T, M>` | push, pop the oldest, fold of the window (`all`) |
| `Subset<B>` | walk all subsets, walk submasks and supermasks, add and remove a member, members as a list |

### R14. Std features compose

Views, keys and operation values pass through every std fn that accepts their trait.

- **Why:** a contestant combines two features they each know and expects it to work.
- **Example:** `zip(a, a.rev)`, `a.maximal(Window(+), ok: ...)`.
- **Enforced by:** `missing` (pairwise composition suite).

### R15. Every common operation elsewhere has a spelling here

Every common operation in the C++ STL, Python builtins and `itertools`, Rust's iterators and collections, and the AtCoder Library has a CP Lang spelling, or is a listed gap.

- **Why:** a contestant arrives knowing one of these, and a missing operation costs a hand-written loop mid-contest.
- **Example:** `itertools.accumulate` is `a.scan(+)`, `collections.Counter` is `xs.tally()`, Rust's `windows(2)` is `a.windows(2)`.
- **Enforced by:** `missing` (comparative inventory table checked against `cpl doc --format json std`).

## Cost

### R16. Cost matches the look of the call

Anything that looks O(1) is O(1), a getter (`@get`) does O(1) work, and an allocating copy looks like one.

- **Why:** the contestant reasons about complexity from the code they read.
- **Example:** `a.rev` is a free view; `List(a.rev)` allocates and looks like it; `factor(x)` is O(sqrt x), `Sieve(n).factor(x)` is O(log x).
- **Enforced by:** `scripts/spec-gate.py std-docs --audit` (`@get` members that do work); `missing` (scaling test per fn).

### R17. Fixed-size windows don't allocate

`a.windows(k)` with a compile-time `k` yields fixed-size values with no allocation; with a runtime `k` it yields slice views.

- **Why:** a sliding window over pairs or triples is a hot loop.
- **Example:** `for (x, y) in a.windows(2):` allocates nothing.
- **Enforced by:** `missing` (allocation counter in the interpreter checked against `cost:`).

## Scope and change

### R18. Any value std gives, a program can name

The prelude exports every std type that a prelude signature names (spec R11.3.2).

- **Why:** a contestant who can get a value can annotate it without hunting for a `use`.
- **Example:** `(w: RingSlice<M, Int>) => w.len` over `a.ring.minimal(...)` needs no `use`.
- **Enforced by:** `spec/conformance/core-11/prelude-exports-signature-types.cp`; `scripts/prelude-signatures.py` (in `scripts/check.sh`).

### R19. std uses only powers a user library has

No std name is special-cased in the compiler; a power std needs is open to user libraries ([std-mechanisms.md](../design/std-mechanisms.md)).

- **Why:** personal libraries get std's ergonomics and speed.
- **Example:** `Subset<B>` is an ordinary type with `impl Index`, not a compiler builtin.
- **Enforced by:** `plan/domains/stdlib/measure.py privilege`, ratcheted by `scripts/debt-ratchet.py` (`prelude_privilege`, `ir_builtin_names`).

### R20. A rename ships its codemod and its error

Renaming a public item ships a `cpl fix` codemod and a checker error on the old name that names the new one.

- **Why:** old solutions keep building, and a contestant with an old habit is told the new spelling.
- **Example:** `K`, `to_idx`, `from_idx` give an error with a fix to `count`, `ord`, `nth`.
- **Enforced by:** corpus ports in `cpl-conform all --check`; `scripts/std-api-snapshot.py` fails a signature change not written to `spec/lib/std-api.snapshot`, and a removed name no `cpl fix` codemod or CHANGELOG.md line names.

### R21. std is lint-clean

std passes `cpl lint` like a contestant's program does, with no hits.

- **Why:** std is read and copied as the model of good code, and a lint that fires on std teaches people to ignore it.
- **Example:** an `L0301` float equality in std is a real comparison bug to fix, or a lint false positive to fix in the lint.
- **Enforced by:** `scripts/std-lint-ratchet.py` (in `scripts/check.sh`): any hit fails.

## Review

Before landing a public std change, confirm each rule above holds or the
change adds the rule it needs. The doc itself meets
[std-docs-checklist.md](std-docs-checklist.md).

## Changes

- 2026-11-23: R21 (std is lint-clean), from the register.
- 2026-11-23: first version, from `std-api.md` and the stdlib register.

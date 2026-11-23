# Containers (`std.bitset`, `std.bag`, `std.interval`, `std.dsu`, `std.heap`, `std.deque`, `std.marks`, `std.hashmap`, `std.seq.Compressed`)

> **0.3.0 status.** This chapter is the family's design survey and API.
> Where it disagrees with the core chapters or the aligned rounds, they
> win: `core/03-numbers-layout.md` (four number types, no `I32`/`U32`,
> `Mod<P>` not `Mint`), `core/05-ownership.md` (constructors own their
> results), `core/07-sequences.md` (capabilities, views keep keys),
> `lib/ops-ranges.md` (`Op`/`Monoid`/laws, `each(...)`, no
> `RangeFenwick`, `MinHeap`/`MaxHeap` as `Heap(by:)`) and
> `lib/sequence-patterns.md`. The stubs in `std/` are the signatures of
> record. Reconciliation of this text is tracked in `delta/`.


Status: design for the CP Lang standard library containers area (`plan/STDLIB.md`
§4.8–4.9). Every type here is ordinary CP Lang code using only the public
language, documented intrinsics, and other stdlib modules — no compiler
privilege.

## The model in one paragraph

Every container is ordinary CP Lang over the public language, chosen by
the operations a solution needs, each with its cost documented. Ordered
data is `SortedSet`, `SortedMap` and `SortedBag`, with cursors and rank;
disjoint runs are `IntervalSet` and `IntervalMap`; connectivity is `Dsu`
and its variants (`ParityDsu`, `PotentialDsu`, `PersistentDsu`); a
priority queue is one `Heap(by:)`, with `IndexedHeap` when priorities
change; `Deque`, `Bitset` and `DynBitset` cover the rest of the common
shapes. `Marks` resets in \(O(1)\) per test, `FlatMap` and `Counter` hash,
and `Compressed` maps raw values onto a domain so later tables are
dense arrays (`core/04-indices.md`). The survey below says which contest
jobs drove each choice; `RATIONALE.md` lists the decisions.

## 1. Contest pattern survey

This survey is grounded in Codeforces Div. 1, AtCoder Grand/ARC/WTF, ICPC World
Finals, KACTL, AtCoder Library (ACL), and `proto/ports/ecnerwala/*_orig.cpp`
plus `research/frictions/*.md`.

### 1.1 Ordered sets, maps, and multisets with rank and cursors

Contestants reach for ordered balanced trees for four distinct jobs:

- **Neighbour queries and local walks.** Given a dynamic set or multiset, find
  the predecessor/successor (`last_lt`, `first_gt`) and walk a few steps
  forward or backward in \(O(1)\) amortized per step. In `cf2066e_orig.cpp`,
  ecnerwala keeps `std::set<std::pair<int, int>>` (adding a unique id to get a
  multiset with \(O(1)\) erase and unique handles) and walks `rbegin()` downwards
  up to `sqrt(M)` steps while also reading `std::prev(it)` and `std::next(it)`
  on every insert and erase. Without \(O(1)\) cursors, each step re-walks from the
  root, turning \(O(q \sqrt{M})\) into \(O(q \sqrt{M} \log n)\) — a 6.11x slowdown and TLE
  in `research/frictions/cf2066e.md`.
- **Order statistics.** `rank(k)` (count of elements `< k`), `nth(i)` (0-based
  k-th smallest), and `count_range(lo, hi)`. KACTL provides `OrderStatisticTree`
  (GNU PBDS `tree_order_statistics_node_update`). Ports `noi22p5`, `noi20p4`,
  `ccc05s5`, `cf2262b` all need order statistics.
- **Monotone predicate search on the tree.** In `cf2262b`, the prefix maxima
  increase in both position and value, so one ordered set answers "first
  maximum whose value exceeds `s`" in \(O(\log n)\) without keeping a second `Map`
  and a `Fenwick` in sync (`research/frictions/cf2262b.md`).
- **Small-to-large merging.** `take()` in \(O(1)\) plus `swap` and draining the
  smaller container into the larger.

Builtin `Set<T>` and `Map<K, V>` cover basic ordered sets/maps (lowering to
`std::set`/`std::map`, or salted `std::unordered_*` when order is never
observed; `for (k, v) in m` walks a map's entries in key order), but they do not carry subtree sizes, cannot answer `rank`/`nth`/
`bsearch_first`/`bsearch_last`, and are not multisets. `std.bag` owns the augmented ordered
containers: `SortedBag<K>` (multiset with rank, cursors, monotone search, and
recycled nodes) and `SortedSet<K>` / `SortedMap<K, V>` when distinct keys
with order statistics and cursors are needed.

### 1.2 Interval partition maps (ODT / Chtholly tree)

Painting ranges of a 1D array or HLD chains with a value/timestamp and
processing every overwritten run `(range, old_val)` is ubiquitous in CF Div. 1
and ICPC (color-overwrite sweeps, HLD path paint as in `cf2237h`, active-segment
maintenance, amortized \(O((n + q) \log n)\) total runs created and destroyed).

In `cf2237h.cp` (`research/frictions/cf2237h.md`), hand-rolling this over
`Map<Int, (r: Int, t: Time)>` takes ~25 lines of fragile `split(l)`, `split(r)`,
`remove(pos)!`, and manual run-length initialization — and forces leaving the
nominal `tree.Pre` domain via `.ord`. Neither KACTL nor ACL ships an interval
partition map; contestants copy-paste the same `split`/`assign` loop and debug
off-by-ones. `std.interval.IntervalMap<D, T>` makes the partition invariant a
type over any domain `D`.

### 1.3 Disjoint-set union (DSU)

ACL ships `dsu` (path compression + union by size + `groups`). KACTL ships
`UnionFind` and `RollbackUF` (undo to `time()`). Across the corpus (56 port
files, including `cf2069f`, `cf2222f`, `cf2237h`, `cf2150f`, `wtf26a`), four
variants recur:

- **Plain DSU with optional rollback (`Dsu<V>`).** Fast path halving
  \(O(alpha(n))\) when `mark()` is never called; switches to union-by-size without
  path compression \(O(\log n)\) once `mark()` is called so `undo_to(snap)` restores
  exact state in \(O(1)\) per undone unite.
- **Payload-carrying DSU (`DsuOf<V, T>`).** Each component carries a `T`
  combined by `join(val_a, val_b)` in argument order `(a, b)` regardless of
  which root union-by-size keeps. This covers both commutative aggregates (sum,
  min, max, first representative in `cf2222f`) and directional skip-DSU
  (`(_, b) => b` in `cf2237h`), with full rollback of merges and `set`s.
- **Bipartite / parity DSU (`ParityDsu<V>`).** Tracks `x[a] xor x[b] == d`,
  contradiction count (`consistent`), and rollback (`mark`/`undo_to`).
- **Weighted / potential DSU (`PotentialDsu<V>`).** Tracks additive difference
  constraints `pot(b) - pot(a) == d` (`diff(a, b) -> ?Int`, contradiction
  detection, plus optional rollback via `Mark`). Promoted from
  `examples/local-lib/ds/potential_dsu.cp`.
- **Companion rollback array (`RollbackArray<D, T>`).** Fixed-domain `[D]T`
  with `Mark`-typed `mark()`/`undo_to(snap)`, promoted from
  `examples/local-lib/ds/rollback_array.cp` and unified onto `Mark`.

### 1.4 Heaps

> Superseded by `std/heap.cp`: one type per shape (`Heap`, `ErasableHeap`,
> `IndexedHeap`, `MeldHeap`) with `by:`; `MinHeap`/`MaxHeap` are aliases of
> `Heap`, `peek` returns an unnamed `?(D, K)`, and `of(xs, by:)` builds in \(O(n)\).

`MinHeap<T>` and `MaxHeap<T>` (chosen by name, never a hidden comparator flag;
mixed tie-breaking via `Desc<T>`) cover standard priority queues, Dijkstra, and
greedy scheduling. Three gaps appear across contests:

- **\(O(n)\) bottom-up construction (`of(xs)`).** Floyd's heapify builds a heap in
  \(O(n)\) instead of \(O(n \log n)\) pushes.
- **Arbitrary removal (`ErasableMinHeap<T>`, `ErasableMaxHeap<T>`).** The
  two-heap lazy-deletion idiom (`live` + `dead`) gives \(O(\log n)\) amortized
  `remove(x)` at binary-heap constant factor (2–3x faster and far less memory
  than a balanced tree when only min/max is needed).
- **Keyed decrease/update-key (`IndexedMinHeap<D, T>`, `IndexedMaxHeap<D, T>`).**
  Over a domain `D`, maps each active key `d: D` to a priority `T` with \(O(\log n)\)
  `set(d, val)`, `remove(d)`, `pop() -> ?(D, T)`, and \(O(1)\) `get(d) -> ?T` /
  `contains(d)`. Avoids stale-entry memory blowup in dense Dijkstra/Prim and
  dynamic scheduling.
- **One pop per key (`Settle<K, P>(best: min)`).** `offer(k, p)` keeps the
  best priority for a key not yet popped and ignores keys already popped;
  `pop() -> ?(K, P)` settles the best key. Dijkstra's frontier, Prim's, and
  "settle answers from the largest down" without a done-flag side array.

### 1.5 Deques and sliding-window queues

`Deque<T>` is a ring buffer with \(O(1)\) amortized ends and \(O(1)\) random access
(`get`, `set`, `first`, `last(k)`). Two classic contest structures sit directly
beside it in `examples/local-lib/ds/`:

- **`Window(op)`** (`std.window`, `lib/ops-ranges.md` §8) — sliding-window minimum,
  maximum or any monoid fold in \(O(1)\) amortized per element; it replaced
  `MonoDeque`. Used in sliding-window DP, convex/Monge DP (`cf2183h`), and range-min sweeps.
- **`TwoStackFold<T>`** — FIFO queue with \(O(1)\) `fold` over any `Monoid<T>`
  without requiring an inverse (gcd, bitwise OR, matrix products).

### 1.6 Bitsets

`Bitset<D>` packs a set of positions of domain `D` into 64-bit words for
\(O(K / 64)\) set algebra and shift DP (`cf2147h`, `cf2187d`, `ioi04p5`, `yac1p4`).
Four friction points surfaced in ports:

- **Knapsack performance trap (`cf2187d`).** The obvious loop
  `for w in ws: reach.union(reach.shifted(w))` allocates a fresh `Bitset` per
  step and is \(O(n \cdot W / 64)\) — 10x slower than the C++ original on 100k equal
  weights. Two fixes belong together: an in-place non-allocating
  `shift_or(k)` / `shift_up(k)` for custom shift DPs, and a stdlib
  `Bitset<D>.subset_sums(weights)` that applies the folklore binary carry (turning
  duplicate weights into \(O(\sqrt{\mathit{sum}})\) items in \(O(K \sqrt{S} / 64)\)).
- **Word-level negative shifts.** `shifted(-k)` currently loops bit-by-bit in
  \(O(K)\) instead of word-by-word in \(O(K / 64)\).
- **Navigation and range queries.** `first`, `last`, `prev(i)` (completing
  `next(i)`), `clear()`, and `count_below(i)` (all \(O(K / 64)\)).
- **Non-mutating single-bit helpers and search.** `with(i)`, `without(i)`, and
  `find(pred)` (from `cf2147h.md`, avoiding `var rest = alive; rest.remove(v)`).

### 1.7 Hash maps and sets

Write `Map` and `Set` for a lookup table; the compiler hashes them when no
walk or ordered query observes their order (§1, decision 231). The rest of
this section is for when hashing must be spelled.

Builtin `HashMap<K, V>` and `HashSet<T>` already emit `std::unordered_map` and
`std::unordered_set` with a salted `splitmix64` hash (`cpl::CpHash`), defeating
Codeforces anti-hash hacks by default. However, `std::unordered_map` allocates
one heap node per entry (high RSS and cache misses). In C++, contestants reach
for `gp_hash_table` or a custom open-addressing flat table (`FlatMap<Int, V>`)
when `n >= 2e5` and hash lookups are the bottleneck.

`std.hashmap` provides `FlatMap<V>` (open-addressing linear
probing over `Int` keys with salted `splitmix64`, power-of-two table, load
factor <= 0.5, \(O(1)\) expected lookup/insert/remove, ~16 bytes/entry vs ~48 bytes
for `std::unordered_map`). It is explicit by name — never silently substituted
for `HashMap`.

### 1.8 Sparse and coordinate-compressed indexing

Three complementary patterns appear across contests:

- **Offline coordinate compression (`std.seq.compress` -> `Compressed<D, T>`).**
  `let (domain C, comp) = compress(xs)` sorts values, deduplicates, and binds
  a fresh domain `C` with the table `comp` (R07.11.5). Today it has
  `get`, `slice`, `len`, `at`, `first_ge`, `last_le`. It lacks `first_gt`,
  `last_lt` (the rest of the 4-name sorted lookup family), `try_at(val) -> ?D`,
  `contains(val) -> Bool`, and `range(lo, hi) -> Range<D>` (turning value bounds
  directly into a domain `Range<D>`).
- **Online bidirectional interning (`std.marks.Interner<K>` or `std.seq`).**
  Assigns dense `0..len` ids to arbitrary keys on the fly (`id`, `get`, `key`,
  `keys`). Promoted from `examples/local-lib/ds/interner.cp`.
- **\(O(1)\)-clear epoch tables (`std.marks.Marks<D>` and `MarkMap<D, T>`).**
  `Marks<D>` already gives an \(O(1)\)-clear visited set over `D` using a 32-bit
  generation stamp. `MarkMap<D, T>` generalizes it to an \(O(1)\)-clear sparse array
  over `D` with a default value `default: T` (`get`, `set`, `contains`,
  `remove`, `clear` all \(O(1)\)), replacing manual epoch arrays in BFS, Dinic, and
  per-query DP resets.

---

## 2. Coherent API design

Every container follows `plan/STDLIB.md` §2:

- Collection verbs: `add`, `remove`, `contains` (`x in c`), `len`, `is_empty`,
  `clear`, `take`.
- Sorted lookup family everywhere a total order exists: `first_ge`, `first_gt`,
  `last_le`, `last_lt`, plus `first`, `last`, `pop_first`, `pop_last`.
- Fallible lookups return `?T`; trapping twins have the short name (`at` vs
  `try_at`, `get` returning `?V` on maps).
- Options are named arguments (`differ:`, `inclusive:`), never `_by` variants.
- Every public item documents its time and memory cost in `///`.
- Domains are preserved end-to-end: `Bitset<D>`, `IntervalMap<D, T>`,
  `Dsu<V>`, `DsuOf<V, T>`, `ParityDsu<V>`, `PotentialDsu<V>`,
  `RollbackArray<D, T>`, `IndexedMinHeap<D, T>`, `IndexedMaxHeap<D, T>`,
  `Marks<D>`, `MarkMap<D, T>`, `Compressed<D, T>`.

### 2.1 `std.bitset`

<!-- spec-only: signatures and costs, not a program -->
```cp
struct Bitset<D>                                // O(K / 64) words, 1 bit/pos
    fn new() -> Bitset<D>                       // O(K / 64)
    fn full() -> Bitset<D>                      // O(K / 64)
    fn from(a: [D]Bool) -> Bitset<D>            // O(K)
    fn contains(self, i: D) -> Bool             // O(1)
    fn add(mut self, i: D)                      // O(1)
    fn remove(mut self, i: D)                   // O(1)
    fn toggle(mut self, i: D)                   // O(1)
    fn with(self, i: D) -> Bitset<D>            // O(K / 64), non-mutating copy
    fn without(self, i: D) -> Bitset<D>         // O(K / 64), non-mutating copy
    fn clear(mut self)                          // O(K / 64)
    @get fn len(self) -> Int                    // O(K / 64)
    @get fn count(self) -> Int                  // O(K / 64)
    @get fn is_empty(self) -> Bool              // O(K / 64), early exit
    @get fn first(self) -> ?D                   // O(K / 64)
    @get fn last(self) -> ?D                    // O(K / 64)
    fn next(self, i: D) -> ?D                   // O(K / 64), first >= i
    fn prev(self, i: D) -> ?D                   // O(K / 64), last < i
    fn count_below(self, i: D) -> Int           // O(K / 64), count < i
    fn union(mut self, o: Bitset<D>)            // O(K / 64)
    fn intersect(mut self, o: Bitset<D>)        // O(K / 64)
    fn subtract(mut self, o: Bitset<D>)         // O(K / 64)
    fn symmetric(mut self, o: Bitset<D>)        // O(K / 64)
    fn bitor(self, o: Bitset<D>) -> Bitset<D>   // O(K / 64)
    fn bitand(self, o: Bitset<D>) -> Bitset<D>  // O(K / 64)
    fn bitxor(self, o: Bitset<D>) -> Bitset<D>  // O(K / 64)
    fn meets(self, o: Bitset<D>) -> Bool        // O(K / 64), early exit
    fn shifted(self, k: Int) -> Bitset<D>       // O(K / 64), signed k
    fn shift_up(mut self, k: Int)               // O(K / 64) in place, k >= 0
    fn shift_or(mut self, k: Int)               // O(K / 64) in place: self |= self << k
    fn each(self) -> yields D                   // O(K / 64 + |self|)
    fn find(self, pred: D -> Bool) -> ?D        // O(K / 64 + visited)

    fn subset_sums(weights: List<Int>) -> Bitset<D>  // static: Bitset<D>.subset_sums(ws)
    // O(K * sqrt(sum) / 64); binary-carry knapsack over 0..D.count
```

<!-- spec-only: signatures and costs, not a program -->
```cp
struct BitTrie<W>                               // multiset of 0..2^W, a count per node
    fn new() -> BitTrie<W>                      // O(1)
    fn of(x: Int) -> BitTrie<W>                 // O(W), traps unless x in 0..2^W
    @get fn len(self) -> Int                    // O(1), with multiplicity
    @get fn is_empty(self) -> Bool              // O(1)
    fn add(mut self, x: Int)                    // O(W)
    fn count(self, x: Int) -> Int               // O(W), 0 when absent
    fn walk(self, x: Int) -> yields (bit: Int, count: Int)  // O(W), path of x from bit W - 1
    fn absorb(mut self, mut other: BitTrie<W>)  // O(min nodes), smaller into larger; empties other
```

Laws: every set operation agrees pointwise with `[D]Bool`; `b.first` equals
`b.next(D.first)`; `b.count_below(i)` equals `count(x < i for x in b.each())`;
`b.shift_or(k)` equals `b | b.shifted(k)` without allocation; `subset_sums`
agrees with brute-force 0/1 subset sums on random small weight multisets.

### 2.2 `std.bag` (augmented ordered multiset, set, and map with cursors)

> Superseded in part: decision 103 replaces `first_ge`/`first_gt`/`last_le`/
> `last_lt`/`count_range` with `range(r)` views, and the monotone searches are
> `first_true`/`last_true` (decision 137, `core/04-indices.md`), not
> `bsearch_first`/`bsearch_last`. `drain(r)` is `drain_range(r)`.

`SortedBag<K>` is backed by a randomized treap over flat arrays with a recycled
free list (eliminating memory leaks on erase, fixing `cf2066e.md` #2) and parent
pointers so a `BagCursor` steps to `prev` / `next` in \(O(1)\) amortized (fixing
`cf2066e.md` #1). Each node stores `(key, pri, cnt, sub, left, right, parent)`.

<!-- spec-only: signatures and costs, not a program -->
```cp
struct BagCursor(id: Int)  // opaque handle into a SortedBag

struct SortedBag<K>                              // O(log n) ops, O(distinct) mem
    fn new() -> SortedBag<K>                     // O(1)
    fn of(xs: List<K>) -> SortedBag<K>           // O(n log n)
    @get fn len(self) -> Int                     // O(1), total with multiplicity
    @get fn distinct(self) -> Int                // O(1), distinct keys
    @get fn is_empty(self) -> Bool               // O(1)
    fn clear(mut self)                           // O(1)
    fn take(mut self) -> SortedBag<K>            // O(1)
    fn add(mut self, x: K) -> Bool               // O(log n); inserts one copy
    fn add_many(mut self, x: K, c: Int)          // O(log n); inserts c >= 0 copies
    fn remove(mut self, x: K) -> Bool            // O(log n); removes one copy
    fn remove_all(mut self, x: K) -> Int         // O(log n); returns removed count
    fn count(self, x: K) -> Int                  // O(log n)
    fn contains(self, x: K) -> Bool              // O(log n)
    @get fn first(self) -> ?K                    // O(log n)
    @get fn last(self) -> ?K                     // O(log n)
    fn pop_first(mut self) -> ?K                 // O(log n)
    fn pop_last(mut self) -> ?K                  // O(log n)
    fn first_ge(self, k: K) -> ?K                // O(log n)
    fn first_gt(self, k: K) -> ?K                // O(log n)
    fn last_le(self, k: K) -> ?K                 // O(log n)
    fn last_lt(self, k: K) -> ?K                 // O(log n)
    fn rank(self, k: K) -> Int                   // O(log n), count strictly < k
    fn count_range(self, lo: K, hi: K) -> Int    // O(log n), count in [lo, hi)
    fn nth(self, i: Int) -> ?K                   // O(log n), 0-based
    fn bsearch_first(self, ok: K -> Bool) -> ?K  // O(log n), monotone ok
    fn bsearch_last(self, ok: K -> Bool) -> ?K   // O(log n), monotone ok
    // Cursor navigation: O(log n) to locate, O(1) amortized to step
    fn cursor_first(self) -> ?BagCursor                  // O(log n)
    fn cursor_last(self) -> ?BagCursor                   // O(log n)
    fn cursor_at(self, k: K) -> ?BagCursor               // O(log n), exact key
    fn cursor_ge(self, k: K) -> ?BagCursor               // O(log n), first >= k
    fn cursor_gt(self, k: K) -> ?BagCursor               // O(log n), first > k
    fn cursor_le(self, k: K) -> ?BagCursor               // O(log n), last <= k
    fn cursor_lt(self, k: K) -> ?BagCursor               // O(log n), last < k
    fn key_of(self, c: BagCursor) -> K                   // O(1)
    fn count_of(self, c: BagCursor) -> Int               // O(1)
    fn next_cursor(self, c: BagCursor) -> ?BagCursor     // O(1) amortized
    fn prev_cursor(self, c: BagCursor) -> ?BagCursor     // O(1) amortized
    fn entries(self) -> yields (key: K, count: Int)      // O(n), ascending
    fn entries_rev(self) -> yields (key: K, count: Int)  // O(n), descending
    fn to_list(self) -> List<K>                          // O(n)
```

`SortedSet<K>` wraps the same treap invariant with uniqueness (`add(x) -> Bool`
is false if already present) and exposes the exact same `first_ge`/`first_gt`/
`range(r)` views, `rank`, `nth`, `bsearch_first`/`bsearch_last`, cursors, and
`each`/`each_rev`. This directly solves `cf2262b.md` (collapsing `Set<I>`,
`Map<Int, I>`, and `Fenwick<I, Int>` into one `SortedSet<I>`).

Why not mutate `SortedBag` through a cursor (`erase(c)`)? Because a treap
`split`/`merge` is already \(O(\log n)\) and invalidates structural paths; keeping
cursors read-only handles that remain valid until the next mutation avoids
dangling-cursor traps while giving \(O(1)\) amortized `prev`/`next` and \(O(1)\)
`entries_rev()` walks (which is exactly what `cf2066e` needs).

Laws: `SortedBag` matches a sorted `List<K>` on every op; `nth(rank(k))` is
`first_ge(k)`; `count_range(lo, hi) == rank(hi) - rank(lo)` for `lo <= hi`;
`entries_rev()` is the exact reverse of `entries()`; after `m` inserts and `m`
removes, internal live node count is 0 and free-list recycling bounds array
length to peak distinct keys.

### 2.3 `std.interval` (ODT / interval partition map)

`IntervalMap<D, T>` maintains a total partition of domain `D` into maximal
half-open runs `[l, r)` mapped to values of `T` (where `T: Eq`). Internally it
stores a `Map<Int, (hi: Int, val: T)>` keyed by `lo.ord` — every position in
`0..D.count` belongs to exactly one entry, and adjacent entries have distinct
values.

<!-- spec-only: signatures and costs, not a program -->
```cp
struct IntervalMap<D, T>                                // O(runs) memory; amortized O(log runs)
    fn new(init: T) -> IntervalMap<D, T>                // O(1); whole domain has value init
    fn from(a: [D]T) -> IntervalMap<D, T>               // O(D.count); compresses maximal runs
    fn at(self, i: D) -> T                              // O(log runs)
    fn run_at(self, i: D) -> (range: Range<D>, val: T)  // O(log runs)
    fn assign(mut self, r: Range<D>, val: T, on_old: (Range<D>, T) -> () = (_, _) => ())
    // Amortized O((1 + overwritten) log runs); returns overwritten runs
    // in ascending order and coalesces adjacent equal runs.
    fn assign_point(mut self, i: D, val: T) -> T
    // O(log runs); assigns i..=i and returns previous value
    fn each(self) -> yields (range: Range<D>, val: T)  // O(runs)
    @get fn runs(self) -> Int                          // O(1), number of maximal runs
```

Why does `assign` take an optional `on_old: (Range<D>, T) -> ()` callback
instead of returning a `List` or being a `yields` generator? A `yields`
generator borrowing `mut self` cannot guarantee the partition is left valid if
the caller `break`s out early (and `cpl_ir::pushed::Feeds` does not push `mut
self` yields), while returning a `List` allocates per call. An optional callback
enforces partition completion, devirtualizes to an inlined template functor in
C++ with zero heap allocation, and defaults to a no-op when overwritten runs are
not needed.

Laws: `IntervalMap<D, T>` agrees at every `i in D` with a brute-force `[D]T`
array under random `assign` and `assign_point` calls; adjacent runs in `each()`
never have equal `val`; the runs partition `D` with no gaps or overlaps.

### 2.4 `std.dsu`

Already ships `Mark`, `Dsu<V>`, `DsuOf<V, T>`, and `ParityDsu<V>`. Additions:

<!-- spec-only: signatures and costs, not a program -->
```cp
// Existing: Mark, Dsu<V>, DsuOf<V, T>, ParityDsu<V>
// New in std.dsu:
struct PotentialDsu<V>                              // O(alpha(n)), O(log n) after mark
    fn new() -> PotentialDsu<V>                     // O(n)
    fn find(mut self, v: V) -> (root: V, pot: Int)  // pot(v) - pot(root)
    fn unite(mut self, a: V, b: V, diff: Int) -> Bool
    // Records pot(b) - pot(a) == diff; false on contradiction
    fn diff(mut self, a: V, b: V) -> ?Int  // pot(b) - pot(a), or None
    fn same(mut self, a: V, b: V) -> Bool
    fn size(mut self, v: V) -> Int
    @get fn components(self) -> Int
    @get fn consistent(self) -> Bool
    fn mark(mut self) -> Mark         // O(1)
    fn undo_to(mut self, snap: Mark)  // O(1) per undone unite

struct RollbackArray<D, T>                       // O(1) get/set/mark, O(changes) undo
    fn new(init: D -> T) -> RollbackArray<D, T>  // O(n), n = D.count; filled is new(_ => v)
    fn get(self, i: D) -> T                      // O(1)
    fn set(mut self, i: D, val: T)               // O(1)
    fn mark(self) -> Mark                        // O(1)
    fn undo_to(mut self, snap: Mark)             // O(1) per undone set
```

Also add `groups()` to `DsuOf<V, T>` so it matches `Dsu<V>`. All five rollback
structures share the single nominal `Mark` type.

Laws: `PotentialDsu` matches a hidden potential array `[V]Int` across random
unites, contradictions, and rollbacks; `RollbackArray` matches full array
snapshots across random `set`/`mark`/`undo_to` sequences.

### 2.5 `std.heap`

> Superseded: see the note in 1.4 and `std/heap.cp`.

<!-- spec-only: signatures and costs, not a program -->
```cp
    // Existing: MinHeap<T>, MaxHeap<T>
    // Added to MinHeap<T> and MaxHeap<T>:
    fn of(xs: List<T>) -> MinHeap<T>  // O(n) Floyd heapify
    fn of(xs: List<T>) -> MaxHeap<T>  // O(n) Floyd heapify

// New in std.heap:
struct ErasableMinHeap<T>                    // O(log n) amortized; two heaps
    fn new() -> ErasableMinHeap<T>           // O(1)
    @get fn len(self) -> Int                 // O(1)
    @get fn is_empty(self) -> Bool           // O(1)
    fn clear(mut self)                       // O(1)
    fn take(mut self) -> ErasableMinHeap<T>  // O(1)
    fn push(mut self, x: T)                  // O(log n)
    fn remove(mut self, x: T)                // O(log n); x must be present
    fn peek(mut self) -> ?T                  // O(log n) amortized
    fn pop(mut self) -> ?T                   // O(log n) amortized

struct ErasableMaxHeap<T>  // same surface, largest first

struct IndexedMinHeap<D, T>                  // O(log n) push/set/remove/pop
    fn new() -> IndexedMinHeap<D, T>         // O(D.count)
    @get fn len(self) -> Int                 // O(1)
    @get fn is_empty(self) -> Bool           // O(1)
    fn clear(mut self)                       // O(len)
    fn contains(self, k: D) -> Bool          // O(1)
    fn get(self, k: D) -> ?T                 // O(1)
    fn set(mut self, k: D, val: T)           // O(log n); insert or update
    fn remove(mut self, k: D) -> ?T          // O(log n)
    @get fn peek(self) -> ?(key: D, val: T)  // O(1)
    fn pop(mut self) -> ?(key: D, val: T)    // O(log n)

struct IndexedMaxHeap<D, T>  // same surface, largest first
```

Note on `ErasableMinHeap.peek`: takes `mut self` because purging lazy-deleted
tops mutates the internal heaps; its doc states this explicitly.

Laws: `MinHeap.of(xs)` pops elements in the exact order of `sorted(xs)`;
`ErasableMinHeap` matches a sorted multiset on random push/remove/peek/pop;
`IndexedMinHeap<D, T>` matches a `[D]?T` array scanned for minimum on random
`set`/`remove`/`pop`.

### 2.6 `std.deque`

<!-- spec-only: signatures and costs, not a program -->
```cp
// Existing: Deque<T> (add set(mut self, i: Int, x: T) and to_list(self) -> List<T>)
// Promoted from examples/local-lib/ds/ (sliding min/max is `Window(op)`, §8 of ops-ranges):
struct TwoStackFold<T>                   // O(1) amortized queue monoid fold
    fn new<M>(op: M) -> TwoStackFold<T>  // op is a Monoid or bare op
    @get fn len(self) -> Int             // O(1)
    @get fn is_empty(self) -> Bool       // O(1)
    fn clear(mut self)                   // O(1)
    fn push(mut self, x: T)              // O(1)
    fn pop(mut self) -> ?T               // O(1) amortized
    @get fn fold(self) -> T              // O(1)
```

### 2.7 `std.marks` and `std.seq.Compressed`

<!-- spec-only: signatures and costs, not a program -->
```cp
// Existing in std.marks: Marks<D>
// New in std.marks:
struct MarkMap<D, T>                     // O(1) get/set/remove/clear
    fn new(default: T) -> MarkMap<D, T>  // O(D.count) once
    fn get(self, v: D) -> ?T             // O(1); None when unset
    fn set(mut self, v: D, x: T)         // O(1)
    fn contains(self, v: D) -> Bool      // O(1)
    fn remove(mut self, v: D) -> ?T      // O(1); the old value
    fn clear(mut self)                   // O(1) epoch bump
    @get fn len(self) -> Int             // O(1)
    @get fn is_empty(self) -> Bool       // O(1)

struct Interner<K>                  // online key <-> 0..len id
    fn new() -> Interner<K>         // O(1)
    @get fn len(self) -> Int        // O(1)
    @get fn is_empty(self) -> Bool  // O(1)
    fn id(mut self, k: K) -> Int    // O(1) expected (HashMap)
    fn get(self, k: K) -> ?Int      // O(log n)
    fn key(self, idx: Int) -> K     // O(1)
    @get fn keys(self) -> List<K>   // O(1)

    // Additions to std.seq.Compressed<D, T>:
    fn try_at(self, val: T) -> ?D             // O(log n)
    fn contains(self, val: T) -> Bool         // O(log n)
    fn first_gt(self, val: T) -> ?D           // O(log n)
    fn last_lt(self, val: T) -> ?D            // O(log n)
    fn range(self, lo: T, hi: T) -> Range<D>  // O(log n); compressed [lo, hi)
```

### 2.8 `std.hashmap` (open-addressing flat hash tables for `Int` keys)

`Map<K, V>` and `Set<T>` are the default lookup tables: they already lower to
the salted hash table when order is never observed (§1). Spell `HashMap<K, V>`
or `HashSet<T>` only to require hashing whatever the program observes, and
then walking one has no defined order. `std.hashmap` adds an explicit open-addressing table for `Int`
keys when constant factor and memory matter:

<!-- spec-only: signatures and costs, not a program -->
```cp
struct FlatMap<V>                                        // O(1) expected; open addressing
    fn new(default_val: V, cap: Int = 16) -> FlatMap<V>  // O(cap)
    @get fn len(self) -> Int                             // O(1)
    @get fn is_empty(self) -> Bool                       // O(1)
    fn clear(mut self)                                   // O(table_size)
    fn contains(self, k: Int) -> Bool                    // O(1) expected
    fn get(self, k: Int) -> ?V                           // O(1) expected
    fn set(mut self, k: Int, v: V)                       // O(1) expected
    fn remove(mut self, k: Int) -> ?V                    // O(1) expected (backward shift)
    @get fn items(self) -> yields (Int, V)               // O(table_size); also keys, values
```

Uses a per-instance random `splitmix64` salt (initialized from `rand`) so
anti-hash inputs cannot force \(O(n)\) probe chains, and backward-shift deletion so
tombstones never degrade performance.

### 2.9 What stays user code

- **Custom lazy treaps / implicit ropes** (`cf2084g2_orig.cpp` slope-trick
  treap with affine-per-index lazy tags, `examples/local-lib/ds/rope.cp`):
  every problem invents its own node payload and pushdown; a generic implicit
  treap belongs in `local-lib`, not a one-size-fits-all stdlib container.
- **Persistent hashed multisets** (`cf2135f`): waits for a general `Arena<T>`
  with 4-byte handles and struct field narrowing (`research/frictions/cf2135f.md`).
- **Offline dynamic connectivity driver (`TimeTree.run` / `offline_over`)**:
  now ships in `std.timetree` using `mark`/`apply`/`answer`/`undo` callbacks
  (zero `TimeStep` allocation, 1 mark per tree node, devirtualized in C++).
- **Two-heap running median**: 6 lines over `MaxHeap` + `MinHeap` (or one
  `SortedBag.nth(len / 2)` when \(O(\log n)\) tree cost is fine).

### 2.10 Core features coordinated with the core-extensions lane

None of these are special-cased in stdlib; all are general core improvements:

- `for x in c` protocol for any struct with `each() -> yields T` (`Bitset`,
  `SortedBag`, `IntervalMap`, `FlatMap`).
- `?.` optional chaining (`top_at.first_gt(c)?.l ?? I.end`, `cf2262b.md`).
- Narrowing range-typed struct fields to 4 bytes (halves `SortedBag` / arena
  node RSS, `cf2135f.md`).
- Scoped `defer` / `undo` for rollback safety across early returns (`cf2222f.md`).
- Default-valued `Map`/`HashMap` and `.keys`/`.values` views (`cf2066c.md`).

---

## 3. Real contest problems written against this API

### 3.1 `cf2066e`: Tropical Season (`SortedBag` with cursors)

Problem: maintain a multiset of barrel volumes under `+ a` / `- a` (`n, q <= 2e5`,
`a <= 1e6`) and answer whether any `k > 0` has `sum(barrels < k) < k`, at least
two barrels `>= k`, and every gap between consecutive barrels `>= k` at least `k`.
Split at `RT = 1000`: walk the top barrels downward in \(O(1)\) per step via
`entries_rev()` for large `k`, and track small gaps via `SortedBag` cursors.

```cp
// Split bad k at RT = sqrt(1e6): large k walks downward from the largest barrel
// in O(1) per step via entries_rev(); small k tracks the rightmost small gap
// of each width 0..=RT.
use std.bag.{SortedBag}

let (n, q): (1..=200_000, 1..=200_000) = read()
const RT = 1000
domain Small = 0..=RT

var cur = SortedBag<Int>()
var small_gaps: [Small]SortedBag<Int> = [Small] _ => SortedBag<Int>()
var last_small: [Small]Int = 0
var cnt_small: [Small]Int = 0
var tot = 0

fn upd_gap(lo: Int, hi: Int, ins: Bool):
    let l = hi - lo
    if l > RT: return
    let d = Small(l)
    if ins: small_gaps[d].add(lo) else: small_gaps[d].remove(lo)
    last_small[d] = small_gaps[d].last ?? 0

fn insert_barrel(v: Int):
    tot += v
    if v <= RT: cnt_small[Small(v)] += 1
    let _ = cur.add(v)
    let c = cur.cursor_at(v)!
    if cur.count_of(c) >= 2:
        upd_gap(v, v, true)
    else:
        let prv = cur.prev_cursor(c)
        let nxt = cur.next_cursor(c)
        if prv != None and nxt != None:
            upd_gap(cur.key_of(prv!), cur.key_of(nxt!), false)
        if prv != None: upd_gap(cur.key_of(prv!), v, true)
        if nxt != None: upd_gap(v, cur.key_of(nxt!), true)

fn has_bad_k() -> Bool:
    var first_step = true
    var cur_val = 0
    var cur_tot = tot
    var cur_gap = 0
    for (nv, mult) in cur.entries_rev():
        if first_step:
            first_step = false
            if mult >= 2: break
            cur_val = nv
            cur_tot = tot - nv
            cur_gap = nv
        else:
            cur_tot -= nv
            cur_gap = min(cur_gap, min(cur_val - nv, nv))
            if cur_tot < cur_gap: return true
            if cur_gap <= RT or mult >= 2: break
            cur_val = nv

    var last_gap = 0
    var cnt_big = cur.len
    var tot_small = 0
    for k in 1..=RT:
        let d = Small(k - 1)
        last_gap max= last_small[d]
        cnt_big -= cnt_small[d]
        tot_small += (k - 1) * cnt_small[d]
        if cnt_big < 2: break
        if last_gap < k and tot_small < k: return true
    false
```

### 3.2 `cf2237h`: Slime and Queries (`IntervalMap` + `DsuOf`)

Problem: tree of `n` vertices, initial slime set, `q` queries moving a cursor
from `cur` to `c` along the tree path, repainting the path with timestamp `t_now`
and greedily consuming older slime buckets. `IntervalMap<tree.Pre, Time>`
replaces the 25-line hand-rolled ODT and eliminates `.ord` escapes.

```cp
// Repaint each HLD chain of the path (cur -> c) with timestamp t_now using
// IntervalMap<tree.Pre, Time>, then greedily drain slime buckets newest-first
// via directional skip-DSU (DsuOf with (_, b) => b).
@!multitest
use std.dsu.{DsuOf}
use std.interval.{IntervalMap}

let (n, m, q): (2..=100_000, 2..=100_000, 1..=100_000) = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let tree = Rooted(Graph.undirected(edges), root: V.first!)

let init_slime: [0..m]V = read()
var cur = init_slime[m - 1]
domain Time = 1..=q + 2

var segs = IntervalMap<tree.Pre, Time>(Time(1))
for v in init_slime: segs.assign_point(tree.pos(v), Time(2))

var bucket = DsuOf<Time, (prv: ?Time, cnt: Int)>(_ => (prv: None, cnt: 0), (_, b) => b)
bucket.set(Time(2), (prv: None, cnt: m))

var tot_ans = 0
var ans: List<Int> = []
for t_now in Time(3)..:
    let c = V.wrap(read<V>() + tot_ans)
    var chunks: List<(t: Time, w: Int)> = []
    for (range, _) in tree.chains(cur, c):
        segs.assign(range, t_now, on_old: (r, t) => chunks.push((t: t, w: r.len)))
    cur = c

    sort(mut chunks, by: x => desc(x.t))
    var (cur_cnt, cur_ans) = (0, 0)
    for chunk in chunks:
        var w = chunk.w
        var it: ?Time = chunk.t
        while let u = it, w > 0:
            let (prv, cnt) = bucket.get(u)
            let take = min(cnt, w)
            cur_cnt += take
            w -= take
            bucket.set(u, (prv: prv, cnt: cnt - take))
            if take == cnt:
                if let p = prv: bucket.unite(u, p) else: it = None
        cur_ans += w
    bucket.set(t_now, (prv: t_now.prev, cnt: cur_cnt))
    tot_ans += cur_ans
    ans.push(tot_ans)
print(ans)
```

### 3.3 `cf2187d`: Cool Problem (`Bitset` + `subset_sums`)

Problem: given string of `0`, `1`, `?` (`n <= 1e5`), compute the sum of all
distinct final values reachable by filling `?`. After the first `?`, each later
`?` contributes an optional half-gap whose total sum is `<= m`, so `subset_sums`
computes all reachable sums in \(O(m \sqrt{m} / 64)\) without manual binary carries.

<!-- spec-only: excerpt of the port; m, half_gaps, end_f, Pos, p0, p1 and Z come from its earlier lines -->
```cp
// Half-gaps sum to at most m, so binary-carry bitset subset_sums finds every
// reachable offset in O(m sqrt(m) / 64) with zero manual carry bookkeeping.
domain Half = 0..=m
let reach = Bitset<Half>.subset_sums(half_gaps)
var cool: Set<Int> = Set()
for i in reach.each():
    cool.add(end_f[Pos(2 * Int(i) + p0)])
    cool.add(end_f[Pos(2 * Int(i) + p1)])
print(sum(Z(v) for v in cool))
```

### 3.4 `cf2262b`: Prefix Maxima Champions (`SortedSet` with `bsearch_first` + `range(r).len`)

Problem: reverse removals into insertions; maintain prefix maxima and count how
many prefix maxima are eaten by champions. `SortedSet<I>` holds the prefix
maxima in one structure, answering both the monotone value search
(`bsearch_first`) and range count (`range(r).len`) in \(O(\log n)\).

<!-- spec-only: fragment over the solution's `I`, `a`, `total`, `c` and `stop` -->
```cp
// Prefix maxima increase in both position and value, so one SortedSet<I>
// answers both "first maximum > s" (bsearch_first) and "how many in c..stop"
// (range(c.l..stop).len), replacing three structures kept in sync by hand.
var tops = SortedSet<I>()
let s = total.sum(c.l..stop)
let reach = if let q = tops.bsearch_first(q => a[q] > s): q.l else: I.end
let eaten = tops.range(c.l..stop).len - 1
```

### 3.5 `cf2069f`: Graph Inclusion (`Dsu<V>` + `Mark` rollback)

Problem: answer `components(A) - components(A + B)` after each edge toggle in
graphs A and B (`n, q <= 4e5`). Two `Dsu<V>` instances share the `Mark`
protocol (`mark()` / `undo_to(snap)`), switching cleanly from \(O(alpha(n))\) to
\(O(\log n)\) rollback on the first mark.

<!-- spec-only: fragment over the solution's `V`, `T`, `Edge` and `ans` -->
```cp
var (a, ab) = (Dsu<V>(), Dsu<V>())
fn solve(span: Range<T>, live: List<Edge>):
    let snap = (a.mark(), ab.mark())
    var rest: List<Edge> = []
    for e in live:
        if e.on.start <= span.start and span.end <= e.on.end:
            if e.in_a: a.unite(e.x, e.y)
            ab.unite(e.x, e.y)
        elif e.on.start < span.end and span.start < e.on.end:
            rest.push(e)
    if span.len == 1: ans[span.first!] = a.components - ab.components
    else:
        let m = span.mid!
        solve(span.start..m.r, rest)
        solve(m.r..span.end, rest)
    a.undo_to(snap.0)
    ab.undo_to(snap.1)
```

---

## 4. Maximalist roadmap

After landing the v1 surface above and rewriting the affected ports (`cf2066e`,
`cf2237h`, `cf2187d`, `cf2262b`, `cf2147h`):

- **B-tree `SortedBag` / `SortedSet` (optional):** `BagNode<K>` now packs
  32-bit `pri`/`left`/`right`/`par` into one 40-byte cache line (`sorted_range`
  `0.61x` vs C++ treap, `sorted_set` `1.26x` vs PBDS `rb_tree_tag`). Moving to
  a cache-friendly B-tree would close the remaining 1.26x treap-depth gap vs
  PBDS red-black trees if needed.

---

## 5. Deferred dream gaps (not in the design)

- **`SortedBag.sum` (1 user):** deferred because only a single dream asks for
  prefix/range sums on `SortedBag`. If a second user appears, the general
  mechanism is a monoid-augmented `SortedBag<K, M: Monoid>` (matching `SegTree`
  / `Fenwick`), not an ad-hoc `.sum` field on `SortedBag<K>`.
- **`PMultiset`:** now in `std.persistent` (no longer deferred).
- **`MinHeap(by:)` (resolved by existing idiom):** `cf2159f.cp` reads cleanly
  with `stdlib/heap.cp`'s documented `(key, payload)` / `(key, desc(v))` entry
  idiom (`var heap = MinHeap<(Int, Seg)>()`), with zero comparator indirection.
- **Dynamic bitset (`DynBitset`):** for problems where the bitset width is
  computed inside a loop without a domain in scope.
- **Persistent `Arena<T>` + `PMultiset<K>`:** typed 4-byte arena handles so
  persistent segment trees, tries, and `cf2135f` need zero raw-int pools.
- **Group-valued `PotentialDsu<V, G>`:** unify `ParityDsu` (Z/2Z) and
  `PotentialDsu` (Z) over any `Group` once `std.algebra` adds `Group`.

---

## 6. Matched C++17 benchmarks (`bench/containers/run.sh`)

All 13 matched C++17 benchmarks (`g++ -std=c++17 -O2`, best of 3 runs,
identical stdout verified): **p50 = 0.95x**, **max = 1.26x** (`sorted_set`).
10 of 13 benches run at `<= 0.98x` matched C++ (`interval_map` `0.56x`,
`sorted_range` `0.61x`, `hash_antihash` `0.64x`, `hash_insert` `0.68x`,
`hash_lookup` `0.79x`, `min_heap` `0.92x`, `flatmap` `0.95x`, `bitset` `0.96x`,
`ordered_map_set` `0.96x`, `monodeque` `0.98x`).

Profiling notes for the three rows above `1.10x`:
- **`dsu` (`1.15x` wall, `30 ms` vs `27 ms` user):** `Dsu<V>` now uses one
  packed 32-bit negative-size array (`4 B/vertex`) and `unchecked:` `find_ord`.
  The remaining ~3 ms is `cpl::umod` + `cpl::pos` in the benchmark's 3M random
  domain conversions (`V(Int((s >> 33) % U64(n)))`).
- **`deque` (`1.14x`, `42.0 ms` vs `36.7 ms`):** raw `List<T>` ring storage
  (eliminating `OptInt` and per-`get` option unwraps, 2x less memory),
  `Deque.reserve(cap)` preallocation, and cached power-of-two `mask` brought
  `deque` down from `1.95x`. The remaining ~5 ms vs `std::deque` is the
  release `assert 0 <= i < n` bounds check and `cpl::umod` on 2M random reads.
- **`sorted_set` (`1.26x`, `112.9 ms` vs `89.5 ms` PBDS `rb_tree_tag`):**
  packing `BagNode<K>` into one 40-byte struct array and switching `add`/`remove`
  to single-pass BST descent + rotation without sibling `pull` cut `sorted_set`
  from `2.55x` (`240.8 ms`) to `1.26x` (`112.9 ms`) and `sorted_range` from
  `1.00x` to `0.61x`. The remaining `1.26x` vs PBDS is the expected randomized
  treap depth (`1.386 log2 n`) vs red-black tree (`~1.05 log2 n`).

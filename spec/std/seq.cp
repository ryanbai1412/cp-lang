//! Sequences: reductions, scans, runs, searching, permutations, monotone stacks, and compression.
//!
//! - Reductions & extrema: `prod`, `is_sorted`, `min`/`max` (with `by:`), `argmax`/`argmin` (with `by:`).
//! - Binary & unimodal search: moved to `std.search` (`bsearch_real`, `ternary_max`/`ternary_min`, `ternary_max_real`/`ternary_min_real`, `parallel_bsearch`).
//! - Sorted lookup: `first_ge`/`first_gt`/`last_le`/`last_lt` (with optional `by:`), returning `?D` on `[D]T`, `?Int` on `List<T>`, and `?(K, V)` on `Map<K, V>`.
//! - Runs, chunking & windows: `runs`, `run_ranges`, `chunk_by`, `buckets`, `windows`, `chunks`, `pairs`, `with_prev`, `cyclic_pairs`, `zip`.
//! - Scans & folds: `scan`, `suffix_scan`, `excl_fold` (cut-indexed `prefix_fold`/`suffix_fold` live in `std.prefix`).
//! - Reordering & selection: `reverse`, `rotate_left`/`rotate_right`, `dedup`/`deduped`, `partition`, `nth_element`, `merge_sorted`.
//! - Permutations: `next_permutation`, `prev_permutation`, `inverse`, `compose`, `perm_cycles`, `perm_pow`, `inversions`.
//! - Monotone stack & sliding extrema: `prev_less`, `next_less`, `prev_greater`, `next_greater`, `sliding_min`, `sliding_max`.
//! - Coordinate compression: `compress(all)` -> `Compressed<D, T>` (`at`, `try_at`, `contains`, `first_ge`, `first_gt`, `last_le`, `last_lt`).

use std.algebra.{Monoid}

/// Every value of `V` once, in some order. `cpl stress` makes and shrinks it as a permutation.
pub type Perm<V> = [V]V

/// Product of all elements of `self` (`T.one` when empty). Works on any `Seq` (`Str`, `List`, `[D]T`, `Range`, views) of `Int`, `Mod<P>`, `F64`. O(n).
pub fn prod<S: Seq, T>(self: S) -> T = todo

/// Whether `self` is non-decreasing (optionally under key `by:`). Works on any `Seq`. O(n).
pub fn is_sorted<S: Seq, T, K>(self: S, by: T -> K = x => x) -> Bool = todo

/// O(n). Reverse `self` in place (or only the subrange `range` when given; prefer `self[l..r].reverse()`).
/// Works on any `IndexedMut` (`List<T>`, `[D]T`, `MutView<T>`, `Str`).
pub fn reverse<S: IndexedMut>(mut self: S, range: ?Range<Int> = none) = todo

/// O(n). Drop each value equal to the one before it, in place: a sorted list keeps one of each.
pub fn dedup<T>(mut self: List<T>) = todo

/// `self` without each value equal to the one before it: the value form of `dedup`, as `sorted` is of `sort`.
/// Works on any `Seq`. O(n) time without hashing.
pub fn deduped<S: Seq, T>(self: S) -> List<T> = todo

/// O(n). The positions holding a new strict maximum, left to right: the first position, then
/// each one greater than everything before it. `record_highs(by: f)` compares `f(x)`.
pub fn record_highs<D, T, K>(self: [D]T, by: T -> K = x => x) -> yields D = todo

/// Distinct values of `self` in first-seen order (optionally keyed by `by:`).
/// Works on any `Seq`. O(n) expected time, O(n) space via `HashSet`. Unlike
/// `dedup` / `deduped` (adjacent duplicates only, O(n) time without hashing),
/// `distinct` drops duplicates anywhere in the sequence.
pub fn distinct<S: Seq, T, K>(self: S, by: T -> K = x => x) -> List<T> = todo

/// The values of `self` that `keep` holds for, then the ones it does not, each in the order they came.
pub fn partition<S: Seq, T>(self: S, keep: T -> Bool) -> (List<T>, List<T>) = todo

/// O(n) expected, O(n log n) worst case, O(1) extra space. Introselect partitioning `self` in place
/// so `self.at(k)` holds the element that would sit at `k` in sorted order (by `by:` or `cmp:`).
pub fn nth_element<S: IndexedMut, T, K>(
    mut self: S,
    k: Int,
    by: T -> K = x => x,
    cmp: ?((T, T) -> Bool) = none,
) = todo

/// O(|a| + |b|), O(|a| + |b|) memory. Stable merge of two sorted sequences (keyed by `by:`),
/// taking from `a` before `b` on ties. Works on any `Indexed` sequences.
pub fn merge_sorted<A: Indexed, B: Indexed, T, K>(
    a: A,
    b: B,
    by: T -> K = x => x,
) -> List<T> = todo

/// O(1) per window. Borrowed length-`k` sliding slice views, left to right. Nothing when `k > self.len`.
pub fn windows<S: Indexed>(self: S, k: Int) -> yields Slice<S> = todo

/// O(1) per chunk. Borrowed non-overlapping slice views of length `k` (the last may be shorter).
pub fn chunks<S: Indexed>(self: S, k: Int) -> yields Slice<S> = todo

/// Adjacent pairs `(prev, x)` of `self`, left to right: `[10, 20, 30].pairs()` gives
/// `(10, 20)` then `(20, 30)`. Works on any `Seq` (`List<T>`, `[D]T`, `Str`, `Range`, views). O(1) per step.
pub fn pairs<S: Seq, T>(self: S) -> yields (T, T) = todo

/// Every element paired with the one before it, starting with `(none, first)`:
/// `[10, 20, 30].with_prev` gives `(none, 10)`, `(10, 20)`, `(20, 30)`. O(1) per step.
pub @get fn with_prev<S: Seq, T>(self: S) -> yields (?T, T) = todo

/// Every element with the one after it, the last with the first: the
/// sides of a polygon, the links of a ring. One pair `(x, x)` for a
/// single element, none for an empty sequence. Works on any `Seq`.
///
///     for (u, v) in poly.cyclic_pairs(): area += cross(u, v)
pub fn cyclic_pairs<S: Seq, T>(self: S) -> yields (T, T) = todo

/// The pairs `(a.at(i), b.at(i))`, as many as the shorter sequence has.
/// Works on any `Indexed` sequences (`List`, `[D]T`, `Str`, `Range`, views). O(min(|a|, |b|)).
pub fn zip<A: Indexed, B: Indexed>(a: A, b: B) = todo

/// The value `m` holds for `k`, after storing `v` there if it held none: `ids.get_or_insert(name, ids.len)`
/// gives each new name the next id. O(log n).
pub fn get_or_insert<K, V>(mut self: Map<K, V>, k: K, v: V) -> V = todo

/// As `get_or_insert`, but the value is made by `f`, which runs only when `m` holds none
/// for `k`: `ids.get_or_insert_with(name, () => next_id())`. O(log n).
pub fn get_or_insert_with<K, V>(mut self: Map<K, V>, k: K, f: fn() -> V) -> V = todo

/// Replace the value under `k` by `f` of it, starting from `init` when `m` holds none:
/// `cnt.update(x, 0, c => c + 1)`. O(log n).
pub fn update<K, V>(mut self: Map<K, V>, k: K, init: V, f: fn(V) -> V) = todo

/// Rotate `self` left by `k` in place (or only the subrange `range` when given):
/// what was at offset `k % len` comes first. Signed Euclidean `k`: negative `k`
/// rotates right. Works on any `IndexedMut` (`List<T>`, `[D]T`, `Str`). O(n), O(1) extra space.
pub fn rotate_left<S: IndexedMut>(mut self: S, k: Int, range: ?Range<Int> = none) = todo

/// Rotate `self` right by `k` in place (or only the subrange `range` when given). Works on any `IndexedMut`. O(n).
pub fn rotate_right<S: IndexedMut>(mut self: S, k: Int, range: ?Range<Int> = none) = todo

/// A new list rotated left by `k`: what was at offset `k % len` comes first.
/// Signed Euclidean `k`: negative `k` rotates right. Works on any `Seq`. O(n).
pub fn rotated_left<S: Seq, T>(self: S, k: Int) -> List<T> = todo

/// A new list rotated right by `k`. Works on any `Seq`. O(n).
pub fn rotated_right<S: Seq, T>(self: S, k: Int) -> List<T> = todo

/// Step `self` to the next arrangement in sorted order, in place. Returns false, leaving `self`
/// sorted ascending, when it was the last one. Works on any `IndexedMut`. O(n), amortized O(1).
pub fn next_permutation<S: IndexedMut>(mut self: S) -> Bool = todo

/// Step `self` to the previous arrangement in sorted order, in place. Returns false, leaving `self`
/// sorted descending, when it was the first (ascending) one. Works on any `IndexedMut`. O(n), amortized O(1).
pub fn prev_permutation<S: IndexedMut>(mut self: S) -> Bool = todo

/// The stretches of equal neighbours (or equal `by:` keys), in order:
/// each a slice of `self`: `"aabccc".runs()` walks `"aa"`, `"b"`, `"ccc"`, and
/// `[r.len for r in s.runs()]` is `[2, 1, 3]`. Each run's `.keys` is its range and
/// `r[r.keys.first!]` its key. O(1) to make, O(n) to walk.
pub fn runs<S: Seq, T, K>(self: S, by: T -> K = x => x) -> yields Slice<S> = todo

/// Group indices of `D` by their key in `K`, preserving order within each bucket. O(|D| + |K|).
pub fn buckets<D, K>(key: [D]K) -> [K]List<D> = todo

/// The distinct values of a coordinate compression, in increasing order, indexed by their
/// own domain `D`. `xs[i]` is the value at index `i`, `xs.at(val)` the index of a value that
/// was compressed, and `xs.first_ge(val)` / `xs.last_le(val)` the nearest index (`?D`) for any value.
pub struct Compressed<D, T>(vals: [D]T):
    /// The value at index `i` (or a `List<T>` slice when given a `Range<D>`). O(1) / O(r.len).
    pub fn get<K>(self, i: K) = todo

    /// The compressed values over a range `r` of `xs.D`, as a list. O(r.len).
    pub fn slice(self, r: Range<D>) -> List<T> = todo

    /// How many distinct values there are.
    pub @get fn len(self) -> Int = todo

    /// The index of `val`, which must be one of the compressed values. O(log n).
    pub fn at(self, val: T) -> D = todo

    /// The index of `val` if it was compressed, or `none`. O(log n).
    pub fn try_at(self, val: T) -> ?D = todo

    /// Whether `val` is one of the compressed values. O(log n).
    pub fn contains(self, val: T) -> Bool = todo

    /// The first index whose value is `>= val`, or `none`. O(log n).
    pub fn first_ge(self, val: T) -> ?D = todo

    /// The first index whose value is `> val`, or `none`. O(log n).
    pub fn first_gt(self, val: T) -> ?D = todo

    /// The last index whose value is `<= val`, or `none`. O(log n).
    pub fn last_le(self, val: T) -> ?D = todo

    /// The last index whose value is `< val`, or `none`. O(log n).
    pub fn last_lt(self, val: T) -> ?D = todo

/// Coordinate compression: `let xs = compress(all)` sorts the values, drops the duplicates
/// and gives them a fresh domain `xs.D`, so an array over them is `[xs.D]Int` and an index of
/// one compression cannot be used in another. Works on any `Seq`. O(n log n).
pub fn compress<S: Seq, T>(all: S) = todo

/// Gives the items of a list a fresh domain, so arrays over them are index-safe:
/// `let (domain Arc, arcs) = [...].with_domain()`. Moves the list, O(1).
pub fn with_domain<T>(self: List<T>) -> (domain I, [I]T) = todo

/// The cycle of a permutation through `from`: `from`, `perm[from]`, ... until it
/// returns to `from`. O(1) to make, O(cycle length) to walk.
pub fn cycle<D>(self: [D]D, from: D) -> yields D = todo

// ---- Range boundary cuts, keyed sorted lookup, ordered Map 4-lookups, and List/Array helpers

/// Left boundary cut of a range (valid even when empty).
pub @get fn l<D>(self: Range<D>) -> Cut<D> = todo

/// Right boundary cut of a range (valid even when empty).
pub @get fn r<D>(self: Range<D>) -> Cut<D> = todo

/// O(1). `(r.l..r.mid, r.mid..r.r)`: two non-empty halves, or `none` when `self.len <= 1`, so
/// `let (left, right) = r.halve() else: base` is the divide-and-conquer base case.
pub fn halve<D>(self: Range<D>) -> ?(Range<D>, Range<D>) = todo

/// O(1). The middle cut: ⌊len/2⌋ elements before it; `r.l` when empty. Its `.r` is the
/// middle element only when `r` is non-empty, so guard with `if r.is_empty` first.
pub @get fn mid<D>(self: Range<D>) -> Cut<D> = todo

/// O(1). The positions in both ranges.
pub fn clip<D>(self: Range<D>, other: Range<D>) -> Range<D> = todo

/// O(log n) calls to `ok`. The first element where `ok`, false then true along `r`, holds;
/// `none` when it never does. Debug builds check `ok` on both sides of the answer.
pub fn first_true<D>(self: Range<D>, ok: D -> Bool) -> ?D = todo

/// O(log n) calls to `ok`. The last element where `ok`, true then false along `r`, holds;
/// `none` when it never does. Debug builds check `ok` on both sides of the answer.
pub fn last_true<D>(self: Range<D>, ok: D -> Bool) -> ?D = todo

/// O(log n) calls to `ok`. The cut where `ok` flips: false then true along `r` by default
/// (`r.l` when it always holds, `r.r` when it never does), true then false with `last: true`
/// (`r.r` when it always holds). `first_true(ok)` is `.r` of it; `last_true(ok)` is
/// `bsearch(ok, last: true).l`.
pub fn bsearch<D>(self: Range<D>, ok: D -> Bool, last: Bool = false) -> Cut<D> = todo

/// The two ends of a range as offsets, `(lo, hi)` with `lo <= hi`: what a range query walks.
/// A range written backwards, `P(3)..P(1)`, stops the program here: no query silently answers
/// as if it were empty. `l..l` is in order and empty.
pub @get fn ends<D>(self: Range<D>) -> (Int, Int) = todo

/// First index in sorted `self` with `by(self[i]) > target`, or `none`. O(log n).
pub fn first_gt<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> ?D = todo

/// Last index in sorted `self` with `by(self[i]) < target`, or `none`. O(log n).
pub fn last_lt<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> ?D = todo

// ---- Permutations, monotone stack, and sliding-window extrema

/// Inverse of bijection `self: [I]V`: `inv[self[i]] == i` for all `i in I`.
/// Traps if `self` is not a bijection. O(|I|).
pub fn inverse<I, V>(self: [I]V) -> [V]I = todo

/// Function composition: `compose(f, g)[a] == g[f[a]]`. O(|A|).
pub fn compose<A, B, C>(f: [A]B, g: [B]C) -> [A]C = todo

/// Disjoint cycle decomposition of permutation `p`, each cycle starting at its smallest element. O(|V|).
pub fn perm_cycles<V>(p: [V]V) -> List<List<V>> = todo

/// `k`-th iterate of permutation `p` (`k >= 0`), computed via cycle decomposition. O(|V|).
pub fn perm_pow<V>(p: [V]V, k: Int) -> [V]V = todo

/// Number of inversions: pairs `(i, j)` with `i < j` and `a[i] > a[j]`. Works on any `Seq`. O(n log n).
pub fn inversions<S: Seq, T>(a: S) -> Int = todo

/// Nearest earlier index `j < i` with `self[j] < self[i]` (or `<=` when `strict: false`), or `none`. O(n).
pub fn prev_less<D, T>(self: [D]T, strict: Bool = true) -> [D]?D = todo

/// Nearest later index `j > i` with `self[j] < self[i]` (or `<=` when `strict: false`), or `none`. O(n).
pub fn next_less<D, T>(self: [D]T, strict: Bool = true) -> [D]?D = todo

/// Nearest earlier index `j < i` with `self[j] > self[i]` (or `>=` when `strict: false`), or `none`. O(n).
pub fn prev_greater<D, T>(self: [D]T, strict: Bool = true) -> [D]?D = todo

/// Nearest later index `j > i` with `self[j] > self[i]` (or `>=` when `strict: false`), or `none`. O(n).
pub fn next_greater<D, T>(self: [D]T, strict: Bool = true) -> [D]?D = todo

/// Minimum of every length-`k` contiguous window of `self` (`n - k + 1` values, empty if `k > n`). Works on any `Seq`. O(n).
pub fn sliding_min<S: Seq, T>(self: S, k: Int) -> List<T> = todo

/// Maximum of every length-`k` contiguous window of `self` (`n - k + 1` values, empty if `k > n`). Works on any `Seq`. O(n).
pub fn sliding_max<S: Seq, T>(self: S, k: Int) -> List<T> = todo

/// Swap two positions of a domain array in place; safely a no-op when `i == j`.
pub fn swap_at<D, T>(mut self: [D]T, i: D, j: D) = todo

/// `self` cut or padded with `fill` to length `n`: `a.resized(size, Z(0))` instead of
/// `[if i < a.len: a[i] else: Z(0) for i in 0..size]`. Works on any `Indexed` sequence. O(n).
pub fn resized<S: Indexed, T>(self: S, n: Int, fill: T) -> List<T> = todo

/// O(1). Pop the last element if `ok` holds for it, else leave `self` alone and give `none`.
/// A monotonic stack is `while let top = st.pop_if(t => t.h >= h): ...`.
pub fn pop_if<T>(mut self: List<T>, ok: T -> Bool) -> ?T = todo

/// O(popped). Pop from the back while `ok` holds for the last element; the popped
/// elements, last first. The back twin of `Deque.pop_front_while`.
pub fn pop_while<T>(mut self: List<T>, ok: T -> Bool) -> List<T> = todo

/// O(n), allocates. A separate copy: `let b = a` moves, `let b = a.clone()` copies.
pub fn clone<T>(self: T) -> T = todo

/// Move all elements out of `self` in O(1), leaving it empty.
pub fn take_all<T>(mut self: List<T>) -> List<T> = todo

/// Insert `x` at index `i` (`0 <= i <= self.len`), shifting elements `i..` right by one. O(n).
pub fn insert<T>(mut self: List<T>, i: Int, x: T) = todo

/// O(n). Remove and return the element at index `i` (`0 <= i < self.len`), shifting `i + 1..` left.
pub fn remove<T>(mut self: List<T>, i: Int) -> T = todo

// =============================================================================
// Spec surface (`= todo` where not yet implemented)
// =============================================================================

// --- Lazy views, slicing, and stream / combinator generators (W4, W6) ---

/// O(1) to make. `s[r]`: a borrowed view of `s` over the range `r`. A domain-indexed source keeps
/// its positions (`a[l..=r][i]` is `a[i]`, and `i` must lie in `l..=r`); a `List` or `Str` slice
/// counts from 0 (`s[2..5][0]` is `s[2]`). Writable through a `var` source; the source's length
/// can't change while it is alive. See `spec/api/2-sequences.md`.
pub struct Slice<S>():
    /// O(1). The slice's indices: a range of the source's positions, or `0..len` for an offset source.
    @get pub fn keys(self) -> Range<Int> = todo
    /// O(1).
    @get pub fn len(self) -> Int = todo
    /// O(1).
    @get pub fn is_empty(self) -> Bool = todo
    /// O(1) per step.
    pub fn iter(self) -> yields Int = todo

/// O(1). The view without its first `k` elements, clamped (traps if `k < 0`). Consuming a view
/// from the front is `w = w.drop(1)`.
pub fn drop<S: Indexed>(self: S, k: Int = 1) -> Slice<S> = todo

/// O(1). The view without its last `k` elements, clamped (traps if `k < 0`).
pub fn drop_last<S: Indexed>(self: S, k: Int = 1) -> Slice<S> = todo

/// O(1). The first `min(k, len)` elements, as a view (traps if `k < 0`).
pub fn take<S: Indexed>(self: S, k: Int) -> Slice<S> = todo

/// O(1). The last `min(k, len)` elements, as a view (traps if `k < 0`).
pub fn take_last<S: Indexed>(self: S, k: Int) -> Slice<S> = todo

/// O(n), allocates. The end of a lazy chain, materialized into the type the context expects
/// (`List`, `Set`, `Map`, `Str`, `[D]T`), else a `List`: `a.filter(p).map(f).collect()`.
/// The same operation as `List(s)`; the postfix form is for chains.
pub fn collect<S: Seq, T>(self: S) -> List<T> = todo

/// O(1) per step. Yield leading elements while `keep(x)` holds, stopping at the first failure.
pub fn take_while<S: Seq, T>(self: S, keep: T -> Bool) -> yields T = todo

/// O(1) per step. Skip leading elements while `drop(x)` holds, then yield the rest.
pub fn drop_while<S: Seq, T>(self: S, drop: T -> Bool) -> yields T = todo

/// O(1) per yielded element. Lazily map each element to a sequence and yield all inner elements in order.
pub fn flat_map<S: Seq, T, U: Seq, V>(self: S, f: T -> U) -> yields V = todo

/// O(1) per yielded element. Lazily concatenate a sequence of sequences.
pub fn flatten<S: Seq, U: Seq, T>(self: S) -> yields T = todo

/// O(1) per step. Yield the `some` values of a sequence of `?T`, skipping `none`.
pub fn present<S: Seq, T>(self: S) -> yields T = todo

/// O(n), O(n) memory. Unwrap every `?T` into a `List<T>`, or return `none` if any element is `none`.
pub fn all_some<S: Seq, T>(self: S) -> ?List<T> = todo

/// O(1) per step. Lazily concatenate `a` then `b` without copying.
pub fn chain<A: Seq, B: Seq, T>(a: A, b: B) -> yields T = todo

/// O(1) per step. Cartesian product `(x, y)` for `x in a`, `y in b` in lexicographic order.
pub fn product<A: Seq, B: Indexed, T, U>(a: A, b: B) -> yields (T, U) = todo

/// O(1) per step. One-shot stream `from, step(from), step(step(from)), ...` until `none`.
pub fn iterate<T>(from: ?T, step: T -> ?T) -> yields T = todo

/// O(1) per step. Every element paired with the next one, ending with `(last, none)`.
pub fn with_next<S: Seq, T>(self: S) -> yields (T, ?T) = todo

/// O(1) per step. Zero-alloc zipped view of equal-length sequences; traps if `a.len != b.len`.
pub fn zip_exact<A: Indexed, B: Indexed, T, U>(a: A, b: B) -> yields (T, U) = todo

/// O(n). Non-overlapping length-`k` chunks, padding the last chunk with `pad` when shorter.
pub fn chunks_padded<S: Indexed, T>(self: S, k: Int, pad: T) -> yields List<T> = todo

// --- Reductions, folds, frequencies, and selection ---

/// O(n). Seedless left fold returning `none` on empty (never traps).
pub fn reduce<S: Seq, T>(self: S, f: (T, T) -> T) -> ?T = todo

/// O(n). Seeded left fold into an arbitrary accumulator `Acc`.
pub fn fold<S: Seq, T, Acc>(self: S, f: (Acc, T) -> Acc, init: Acc) -> Acc = todo

/// O(n). Maximum element (under `by:`) together with its multiplicity, or `none` when empty.
pub fn max_count<S: Seq, T, K>(self: S, by: T -> K = x => x) -> ?(val: T, count: Int) = todo

/// O(n). Minimum element (under `by:`) together with its multiplicity, or `none` when empty.
pub fn min_count<S: Seq, T, K>(self: S, by: T -> K = x => x) -> ?(val: T, count: Int) = todo

/// O(n) expected, O(distinct) memory. Frequency map of keys `by(x)` across `self`.
pub fn tally<S: Seq, T, K>(self: S, by: T -> K = x => x) -> HashMap<K, Int> = todo

/// O(n) expected, O(n) memory. Group elements by arbitrary hashable key (companion to domain-indexed `buckets`).
pub fn group<S: Seq, T, K>(self: S, by: T -> K) -> HashMap<K, List<T>> = todo

/// O(n) expected, O(k) memory. `k` smallest elements under `by:` (unordered; ties to earlier index).
pub fn min_k<S: Seq, T, K>(self: S, k: Int, by: T -> K = x => x) -> List<T> = todo

/// O(n) expected, O(k) memory. `k` largest elements under `by:` (unordered; ties to earlier index).
pub fn max_k<S: Seq, T, K>(self: S, k: Int, by: T -> K = x => x) -> List<T> = todo

/// O(n). Unstable in-place partition moving `keep(x)` elements to `0..mid`; returns `mid`.
pub fn partition_in_place<S: IndexedMut, T>(mut self: S, keep: T -> Bool) -> Int = todo

/// O(n), O(n) memory. Stable in-place partition preserving relative order in both halves; returns `mid`.
pub fn stable_partition_in_place<S: IndexedMut, T>(mut self: S, keep: T -> Bool) -> Int = todo

// --- Differences and cut-indexed forms (decision 124) ---

/// O(n), O(n) memory. `[a[0], a[1] - a[0], ...]` (inverse of `scan(+)`).
pub fn adjacent_diff<S: Seq, T>(self: S) -> List<T> = todo

/// O(n), O(n) memory. Consecutive differences `[a[1] - a[0], a[2] - a[1], ...]` (length `max(0, n - 1)`).
pub fn diffs<S: Seq, T>(self: S) -> List<T> = todo

/// O(1). Explicit piece `Range<D>` between cuts `lo` and `hi` (explicit twin of `lo..hi` in decision 124; traps if `lo > hi`).
pub fn span<D>(lo: Cut<D>, hi: Cut<D>) -> Range<D> = todo

// --- Sorted lookup, cut partition point, sorted set ops, and linear-time sorts ---

/// O(log n). Cut between the false prefix and true suffix of monotone `ok` on `self` (total; never `none`).
pub fn partition_point<D, T>(self: [D]T, ok: T -> Bool) -> Cut<D> = todo

/// O(log n). Contiguous range of elements in sorted `self` with `by(x) == target` (empty at insertion cut if absent).
pub fn equal_range<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> Range<D> = todo

/// O(|a| + |b|), O(|a| + |b|) memory. Sorted deduplicated union of two sorted sequences.
pub fn sorted_union<A: Indexed, B: Indexed, T>(a: A, b: B) -> List<T> = todo

/// O(|a| + |b|), O(|a| + |b|) memory. Sorted intersection of two sorted sequences (preserving minimum multiplicity).
pub fn sorted_intersect<A: Indexed, B: Indexed, T>(a: A, b: B) -> List<T> = todo

/// O(|a| + |b|), O(|a| + |b|) memory. Sorted elements in `a` not in `b`.
pub fn sorted_diff<A: Indexed, B: Indexed, T>(a: A, b: B) -> List<T> = todo

/// O(N log k), O(N) memory. Merge `k` sorted sequences via min-heap.
pub fn kway_merge<S: Indexed, T>(lists: List<S>) -> List<T> = todo

/// O(n), O(n) memory. Stable LSD radix sort in place by integer key `by:` (signed `Int` supported).
pub fn radix_sort<S: IndexedMut, T>(mut self: S, by: T -> Int = x => x) = todo

/// O(n + |K|), O(n + |K|) memory. Stable counting sort in place by domain or bounded-range key `by:`.
pub fn counting_sort<S: IndexedMut, T, K>(mut self: S, by: T -> K) = todo

/// O(n) expected, O(n) memory. Whether `a` and `b` have the same multiset of elements.
pub fn is_permutation<A: Seq, B: Seq, T>(a: A, b: B) -> Bool = todo

/// O(|self| + |needle|). Whether `needle` occurs as a subsequence of `self`.
pub fn is_subsequence<S: Seq, N: Indexed, T>(self: S, needle: N) -> Bool = todo

// --- Permutations, combinatorics, subsequences, and monotone-stack trees ---

/// O(n log n), O(n) memory. Lexicographic rank of permutation `p` in `0..|V|!` (`|V| <= 20`).
pub fn perm_rank<V>(p: [V]V) -> Int = todo

/// O(n log n), O(n) memory. The `rank`-th lexicographic permutation of `V` (`0 <= rank < |V|!`).
pub fn perm_unrank<V>(rank: Int) -> [V]V = todo

/// O(n), O(n) memory. Sign/parity of permutation `p` (`+1` even, `-1` odd).
pub fn perm_sign<V>(p: [V]V) -> Int = todo

/// O(n), O(n) memory. Order of permutation `p` (LCM of disjoint cycle lengths).
pub fn perm_order<V>(p: [V]V) -> Int = todo

/// O(|I| + |V|), O(|I| + |V|) memory. Inverse of partial injection `self: [I]?V -> [V]?I` (traps on duplicate non-`none` images).
pub fn partial_inverse<I, V>(self: [I]?V) -> [V]?I = todo

/// O(|I| + |V|), O(|I| + |V|) memory. Preimage buckets: for each `v in V`, all `i in I` with `self[i] == some(v)` in `I` order.
pub fn preimages<I, V>(self: [I]?V) -> [V]List<I> = todo

/// O(k) per combination. All `k`-element combinations in lexicographic index order (`k = 2` is unordered pairs).
pub fn combinations<S: Indexed, T>(self: S, k: Int) -> yields List<T> = todo

/// O(1) per pair. Zero-alloc unordered index pairs `(i, j)` with `i < j` over `self: Range<D>`.
pub fn index_pairs<D>(self: Range<D>) -> yields (D, D) = todo

/// O(k) per permutation. All `k`-permutations of `self` (default `k = self.len`).
pub fn permutations<S: Indexed, T>(self: S, k: ?Int = none) -> yields List<T> = todo

/// O(1). Steerable item yielded by `subsequences` (decision 126, W6).
pub struct SubseqStep<T>(items: List<T>):
    /// O(1). Skip extending this prefix in the `subsequences` search tree.
    pub fn prune(mut self) = todo

/// O(k) per subsequence. All subsequences of `self` in lex order; supports `step.prune()` (decision 126, W6).
pub fn subsequences<S: Indexed, T>(self: S) -> yields SubseqStep<T> = todo

/// O(n log n), O(n) memory. Longest increasing (or non-decreasing when `strict: false`) subsequence indices in `D`.
pub fn lis<D, T>(self: [D]T, strict: Bool = true) -> List<D> = todo

/// O(|a| * |b|), O(|a| * |b|) memory. Longest common subsequence of `a` and `b` as matched `(i, j)` index pairs.
pub fn lcs<A: Indexed, B: Indexed, T>(a: A, b: B) -> List<(Int, Int)> = todo

/// O(n), O(n) memory. Min-heap (or max-heap when `min_root: false`) Cartesian tree in one monotone-stack pass.
pub fn cartesian_tree<D, T>(self: [D]T, min_root: Bool = true) -> (root: ?D, left: [D]?D, right: [D]?D, parent: [D]?D) = todo

/// O(n), O(n) memory. Largest axis-aligned rectangle under `heights` and its achieving `Range<D>`.
pub fn max_histogram<D>(heights: [D]Int) -> (area: Int, range: Range<D>) = todo

// --- Incremental / online sequence structures (with mark/undo_to rollback for tree DFS) ---

/// O(1) construction; O(log n) push, O(1) amortized rollback. Online patience-sorting LIS tracker.
pub struct OnlineLis<T>(strict: Bool = true):
    /// O(log n). Append `x` and return the length of the longest increasing subsequence ending at `x`.
    pub fn push(mut self, x: T) -> Int = todo
    /// O(1). Current global LIS length.
    @get pub fn len(self) -> Int = todo
    /// O(1). Whether no elements have been pushed.
    @get pub fn is_empty(self) -> Bool = todo
    /// O(1). Reset all state.
    pub fn clear(mut self) = todo
    /// O(n), O(n) memory. Reconstruct one global LIS as 0-based push indices.
    pub fn reconstruct(self) -> List<Int> = todo
    /// O(1). Save a rollback checkpoint for DFS-on-tree LIS.
    pub fn mark(self) -> Int = todo
    /// O(1) amortized. Roll back all `push`es since checkpoint `m`.
    pub fn undo_to(mut self, m: Int) = todo

/// O(1) construction; O(1) amortized push and rollback. Online monotone stack returning nearest dominating index.
pub struct MonoStack<D, T>(keep_smaller: Bool = true, strict: Bool = true):
    /// O(1) amortized. Push `(i, x)`, pop dominated entries, and return the nearest dominating index (or `none`).
    pub fn push(mut self, i: D, x: T) -> ?D = todo
    /// O(1). Current top `(index, val)`, or `none` when empty.
    @get pub fn top(self) -> ?(index: D, val: T) = todo
    /// O(1). Current stack depth.
    @get pub fn len(self) -> Int = todo
    /// O(1). Whether the stack is empty.
    @get pub fn is_empty(self) -> Bool = todo
    /// O(1). Empty the stack and rollback log.
    pub fn clear(mut self) = todo
    /// O(1). Save a rollback checkpoint for DFS-on-tree monotone stacks.
    pub fn mark(self) -> Int = todo
    /// O(1) amortized. Roll back all `push`es since checkpoint `m`.
    pub fn undo_to(mut self, m: Int) = todo

@test
fn scan_folds_every_prefix() = todo

@test
fn sorted_lookup_and_linear_search() = todo

@test
fn reorder_and_permutations_in_place() = todo

@test
fn compress_gives_the_values_their_own_domain() = todo

@test
fn min_max_by_tie_breaking() = todo

@test
fn range_bounds_and_keyed_sorted_lookup() = todo

@test
fn list_insert_remove_take_and_swap_at() = todo

@test
fn map_entry_remove_and_four_lookups() = todo

@test
fn deduped_and_partition_leave_the_list_alone() = todo

@test
fn get_or_insert_stores_only_what_is_missing() = todo

@test
fn get_or_insert_with_makes_a_value_only_when_missing() = todo

@test
fn update_starts_from_init_and_then_from_the_value() = todo

@test
fn runs_with_by_groups_neighbours_with_the_same_key() = todo

@test
fn excl_fold_folds_all_but_one() = todo

@test
fn windows_and_zip_walk_in_step() = todo

@test
fn pairs_and_with_prev_yield_neighbours() = todo

@test
fn cyclic_pairs_close_the_ring() = todo

/// The index of the first largest value (optionally under key `by:`), or `none` when empty:
/// `[3, 9, 2, 9].argmax` is `some(1)` and `words.argmax(by: w => w.len)` is the index of the
/// longest word. It is an index of `self`, so `a[a.argmax!]` is the largest value of a
/// `List<T>`, a `[1..=n]T` or a `[V]T` alike. O(n).
pub fn argmax<D, T, K>(self: [D]T, by: T -> K = x => x) -> ?D = todo

/// The index of the first smallest value (optionally under key `by:`), or `none` when empty. O(n).
pub fn argmin<D, T, K>(self: [D]T, by: T -> K = x => x) -> ?D = todo

@test
fn argmax_and_argmin_give_the_first_best_index() = todo

@test
fn prod_and_is_sorted_cover_sequences() = todo

@test
fn nth_element_merge_sorted_and_rotate_right() = todo

@test
fn chunk_by_run_ranges_buckets_and_chunks() = todo

@test
fn suffix_scan_matches_right_to_left_fold() = todo

@test
fn first_gt_and_last_lt_match_linear_scan() = todo

@test
fn permutations_and_inversions() = todo

@test
fn monotone_stack_and_sliding_extrema() = todo

@test
fn seq_and_indexed_traits_unify_sequences_and_views() = todo

@test
fn distinct_with_by_key_keeps_first_seen() = todo

@test
fn scan_prod_and_rotate_method_syntax() = todo

/// O(1). The pair the other way round: `(a, b).rev` is `(b, a)`, and labels stay in place,
/// so an edge `(u: a, v: b).rev` is `(u: b, v: a)`.
pub @get fn rev<A>(self: (A, A)) -> (A, A) = todo

//! Sequences: reductions, scans, runs, searching, permutations, monotone stacks, and compression.
//!
//! - Reductions & extrema: `prod`, `is_sorted`, `min`/`max` (with `by:`), `argmax`/`argmin` (with `by:`).
//! - Binary & unimodal search: moved to `std.search` (`bsearch_first`, `bsearch_last`, `bsearch_real`, `ternary_max`/`ternary_min`, `ternary_max_real`/`ternary_min_real`, `parallel_bsearch`).
//! - Sorted lookup: `first_ge`/`first_gt`/`last_le`/`last_lt` (with optional `by:`), returning `?D` on `[D]T`, `?Int` on `List<T>`, and `?(K, V)` on `Map<K, V>`.
//! - Runs, chunking & windows: `runs`, `run_ranges`, `chunk_by`, `buckets`, `windows`, `chunks`, `pairs`, `with_prev`, `cyclic_pairs`, `zip`.
//! - Scans & folds: `scan`, `suffix_scan`, `excl_fold` (cut-indexed `prefix_fold`/`suffix_fold` live in `std.prefix`).
//! - Reordering & selection: `reverse`, `rotate_left`/`rotate_right`, `dedup`/`deduped`, `partition`, `nth_element`, `merge_sorted`.
//! - Permutations: `next_permutation`, `prev_permutation`, `inverse`, `compose`, `perm_cycles`, `perm_pow`, `inversions`.
//! - Monotone stack & sliding extrema: `prev_less`, `next_less`, `prev_greater`, `next_greater`, `sliding_min`, `sliding_max`.
//! - Coordinate compression: `compress(all)` -> `Compressed<D, T>` (`at`, `try_at`, `contains`, `first_ge`, `first_gt`, `last_le`, `last_lt`).

use std.algebra.{Monoid}
use std.search.{bsearch_first, bsearch_last}

/// Every value of `V` once, in some order. `cpl stress` makes and shrinks it as a permutation.
pub type Perm<V> = [V]V

/// O(n) calls of `f`. Every prefix folded with `f`: `[a0, f(a0, a1), ...]`, indexed as `self` is.
/// Spec target also accepts `init: Acc`, returning `[D.bounds]Acc` (cut-indexed exclusive scan;
/// see spec-syntax gap for optional `init:` refining return domain from `[D]T` to `[D.bounds]Acc`).
pub fn scan<D, T>(self: [D]T, f: (T, T) -> T) -> [D]T:
    var out = self
    out.scan_in_place(f)
    out

/// O(n) calls of `f`. Every suffix folded right-to-left with `f`: `out[i] = f(self[i], out[i + 1])`.
/// Spec target also accepts `init: Acc`, returning `[D.bounds]Acc` (cut-indexed exclusive suffix scan).
pub fn suffix_scan<D, T>(self: [D]T, f: (T, T) -> T) -> [D]T:
    var out = self
    out.suffix_scan_in_place(f)
    out

/// Fold every prefix of `self` in place with `f`: `self.at(i) = f(self.at(i - 1), self.at(i))`.
/// Works on any `IndexedMut` (`List<T>`, `[D]T`, `Str`). Zero allocations. O(n) calls of `f`.
pub fn scan_in_place<S: IndexedMut, T>(mut self: S, f: (T, T) -> T):
    if self.len <= 1: return
    var acc = self.at(0)
    for i in 1..self.len:
        acc = f(acc, self.at(i))
        self.set_at(i, acc)

/// Fold every suffix of `self` right-to-left in place with `f`: `self.at(i) = f(self.at(i), self.at(i + 1))`.
/// Works on any `IndexedMut` (`List<T>`, `[D]T`, `Str`). Zero allocations. O(n) calls of `f`.
pub fn suffix_scan_in_place<S: IndexedMut, T>(mut self: S, f: (T, T) -> T):
    if self.len <= 1: return
    var acc = self.at(self.len - 1)
    for i in (0..self.len - 1).rev:
        acc = f(self.at(i), acc)
        self.set_at(i, acc)

/// Product of all elements of `self` (`T.one` when empty). Works on any `Seq` (`Str`, `List`, `[D]T`, `Range`, views) of `Int`, `Mod<P>`, `F64`. O(n).
pub fn prod<S: Seq, T>(self: S) -> T:
    var out = T.one
    for x in self: out *= x
    out

/// Whether `self` is non-decreasing (optionally under key `by:`). Works on any `Seq`. O(n).
pub fn is_sorted<S: Seq, T, K>(self: S, by: T -> K = x => x) -> Bool:
    var prev: ?K = none
    for x in self:
        let k = by(x)
        if let p = prev, k < p: return false
        prev = k
    true

/// O(n). Reverse `self` in place (or only the subrange `range` when given; prefer `self[l..r].reverse()`).
/// Works on any `IndexedMut` (`List<T>`, `[D]T`, `MutView<T>`, `Str`).
pub fn reverse<S: IndexedMut>(mut self: S, range: ?Range<Int> = none):
    var lo = 0
    var hi = self.len
    if let r = range:
        if r.is_empty: return
        lo = r.first!
        hi = r.last! + 1
    var (i, j) = (lo, hi - 1)
    while i < j:
        self.swap_idx(i, j)
        i += 1
        j -= 1

/// O(n). Drop each value equal to the one before it, in place: a sorted list keeps one of each.
pub fn dedup<T>(mut self: List<T>):
    var kept = 0
    for i in 0..self.len:
        if kept == 0 or self[kept - 1] != self[i]:
            self[kept] = self[i]
            kept += 1
    self.truncate(kept)

/// `self` without each value equal to the one before it: the value form of `dedup`, as `sorted` is of `sort`.
/// Works on any `Seq`. O(n) time without hashing.
pub fn deduped<S: Seq, T>(self: S) -> List<T>:
    var out: List<T> = []
    for x in self:
        if out.is_empty or out.last! != x:
            out.push(x)
    out

/// Distinct values of `self` in first-seen order (optionally keyed by `by:`).
/// Works on any `Seq`. O(n) expected time, O(n) space via `HashSet`. Unlike
/// `dedup` / `deduped` (adjacent duplicates only, O(n) time without hashing),
/// `distinct` drops duplicates anywhere in the sequence.
pub fn distinct<S: Seq, T, K>(self: S, by: T -> K = x => x) -> List<T>:
    var seen = HashSet<K>()
    var out: List<T> = []
    for x in self:
        if seen.add(by(x)):
            out.push(x)
    out

/// The values of `self` that `keep` holds for, then the ones it does not, each in the order they came.
pub fn partition<S: Seq, T>(self: S, keep: T -> Bool) -> (List<T>, List<T>):
    var (yes, no): (List<T>, List<T>) = ([], [])
    for x in self:
        if keep(x): yes.push(x)
        else: no.push(x)
    (yes, no)

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
) -> List<T>:
    var out: List<T> = []
    var (i, j) = (0, 0)
    while i < a.len and j < b.len:
        if by(b.at(j)) < by(a.at(i)):
            out.push(b.at(j))
            j += 1
        else:
            out.push(a.at(i))
            i += 1
    while i < a.len:
        out.push(a.at(i))
        i += 1
    while j < b.len:
        out.push(b.at(j))
        j += 1
    out

/// O(1) per window. Borrowed length-`k` sliding slice views, left to right. Nothing when `k > self.len`.
pub fn windows<S: Indexed, T>(self: S, k: Int) -> yields View<T> = todo

/// O(1) per chunk. Borrowed non-overlapping slice views of length `k` (the last may be shorter).
pub fn chunks<S: Indexed, T>(self: S, k: Int) -> yields View<T> = todo

/// Adjacent pairs `(prev, x)` of `self`, left to right: `[10, 20, 30].pairs()` gives
/// `(10, 20)` then `(20, 30)`. Works on any `Seq` (`List<T>`, `[D]T`, `Str`, `Range`, views). O(1) per step.
pub fn pairs<S: Seq, T>(self: S) -> yields (T, T):
    var prev: ?T = none
    for x in self:
        if let p = prev: yield (p, x)
        prev = x

/// Every element paired with the previous one, starting with `(none, first)`:
/// `[10, 20, 30].with_prev()` gives `(none, 10)`, `(10, 20)`, `(20, 30)`. Works on any `Seq`. O(1) per step.
pub fn with_prev<S: Seq, T>(self: S) -> yields (?T, T):
    var prev: ?T = none
    for x in self:
        yield (prev, x)
        prev = x

/// Every element with the one after it, the last with the first: the
/// sides of a polygon, the links of a ring. One pair `(x, x)` for a
/// single element, none for an empty sequence. Works on any `Seq`.
///
///     for (u, v) in poly.cyclic_pairs(): area += cross(u, v)
pub fn cyclic_pairs<S: Seq, T>(self: S) -> yields (T, T):
    var first_val: ?T = none
    var prev: ?T = none
    for x in self:
        if let p = prev: yield (p, x)
        else: first_val = x
        prev = x
    if let p = prev, let f = first_val:
        yield (p, f)

/// The pairs `(a.at(i), b.at(i))`, as many as the shorter sequence has.
/// Works on any `Indexed` sequences (`List`, `[D]T`, `Str`, `Range`, views). O(min(|a|, |b|)).
pub fn zip<A: Indexed, B: Indexed>(a: A, b: B) = [(a.at(i), b.at(i)) for i in 0..min(a.len, b.len)]

/// The value `m` holds for `k`, after storing `v` there if it held none: `ids.get_or_insert(name, ids.len)`
/// gives each new name the next id. O(log n).
pub fn get_or_insert<K, V>(mut self: Map<K, V>, k: K, v: V) -> V:
    if not (k in self): self[k] = v
    self[k]

/// As `get_or_insert`, but the value is made by `f`, which runs only when `m` holds none
/// for `k`: `ids.get_or_insert_with(name, () => next_id())`. O(log n).
pub fn get_or_insert_with<K, V>(mut self: Map<K, V>, k: K, f: fn() -> V) -> V:
    if not (k in self): self[k] = f()
    self[k]

/// Replace the value under `k` by `f` of it, starting from `init` when `m` holds none:
/// `cnt.update(x, 0, c => c + 1)`. O(log n).
pub fn update<K, V>(mut self: Map<K, V>, k: K, init: V, f: fn(V) -> V):
    self[k] = f(self.get(k) ?? init)

/// Rotate `self` left by `k` in place (or only the subrange `range` when given):
/// what was at offset `k % len` comes first. Signed Euclidean `k`: negative `k`
/// rotates right. Works on any `IndexedMut` (`List<T>`, `[D]T`, `Str`). O(n), O(1) extra space.
pub fn rotate_left<S: IndexedMut>(mut self: S, k: Int, range: ?Range<Int> = none):
    var lo = 0
    var hi = self.len
    if let r = range:
        if r.is_empty: return
        lo = r.first!
        hi = r.last! + 1
    let len = hi - lo
    if len <= 1: return
    let s = k % len
    if s == 0: return
    reverse(mut self, range: lo..lo + s)
    reverse(mut self, range: lo + s..hi)
    reverse(mut self, range: lo..hi)

/// Rotate `self` right by `k` in place (or only the subrange `range` when given). Works on any `IndexedMut`. O(n).
pub fn rotate_right<S: IndexedMut>(mut self: S, k: Int, range: ?Range<Int> = none):
    rotate_left(mut self, -k, range: range)

/// A new list rotated left by `k`: what was at offset `k % len` comes first.
/// Signed Euclidean `k`: negative `k` rotates right. Works on any `Seq`. O(n).
pub fn rotated_left<S: Seq, T>(self: S, k: Int) -> List<T>:
    let n = self.len
    if n <= 1: return [x for x in self]
    let s = k % n
    if s == 0: return [x for x in self]
    var out: List<T> = []
    out.reserve(n)
    var head: List<T> = []
    head.reserve(s)
    var idx = 0
    for x in self:
        if idx < s: head.push(x)
        else: out.push(x)
        idx += 1
    for x in head: out.push(x)
    out

/// A new list rotated right by `k`. Works on any `Seq`. O(n).
pub fn rotated_right<S: Seq, T>(self: S, k: Int) -> List<T>:
    rotated_left(self, -k)

/// Step `self` to the next arrangement in sorted order, in place. Returns false, leaving `self`
/// sorted ascending, when it was the last one. Works on any `IndexedMut`. O(n), amortized O(1).
pub fn next_permutation<S: IndexedMut>(mut self: S) -> Bool:
    var i = self.len - 2
    while i >= 0 and not (self.at(i) < self.at(i + 1)): i -= 1
    if i < 0:
        reverse(mut self)
        return false
    var j = self.len - 1
    while not (self.at(i) < self.at(j)): j -= 1
    self.swap_idx(i, j)
    var (l, r) = (i + 1, self.len - 1)
    while l < r:
        self.swap_idx(l, r)
        l += 1
        r -= 1
    true

/// Step `self` to the previous arrangement in sorted order, in place. Returns false, leaving `self`
/// sorted descending, when it was the first (ascending) one. Works on any `IndexedMut`. O(n), amortized O(1).
pub fn prev_permutation<S: IndexedMut>(mut self: S) -> Bool:
    var i = self.len - 2
    while i >= 0 and not (self.at(i + 1) < self.at(i)): i -= 1
    if i < 0:
        reverse(mut self)
        return false
    var j = self.len - 1
    while not (self.at(j) < self.at(i)): j -= 1
    self.swap_idx(i, j)
    var (l, r) = (i + 1, self.len - 1)
    while l < r:
        self.swap_idx(l, r)
        l += 1
        r -= 1
    true

/// The stretches of equal neighbours (or equal `by:` keys), in order:
/// `"aabccc".runs()` is `[('a', 2), ('b', 1), ('c', 3)]`, and
/// `["ab", "ac", "b"].runs(by: s => s[0])` is `[('a', 2), ('b', 1)]`.
/// Works on any `Seq` (`Str`, `List<T>`, `[D]T`, `Range<I>`, and O(1) views) without conversion. O(n).
pub fn runs<S: Seq, T, K>(self: S, by: T -> K = x => x) -> List<(val: K, len: Int)>:
    var out: List<(val: K, len: Int)> = []
    for x in self:
        let k = by(x)
        if out.len > 0 and out[out.len - 1].val == k:
            out[out.len - 1] = (val: k, len: out[out.len - 1].len + 1)
        else:
            out.push((val: k, len: 1))
    out

/// Partition `self` into maximal contiguous `Range<D>` segments where `same(a, b)` holds on
/// every adjacent pair. Zero allocations, O(n) calls of `same`.
pub fn chunk_by<D, T>(self: [D]T, same: (T, T) -> Bool) -> yields Range<D>:
    var start: ?D = none
    var prev: ?D = none
    for i in D:
        if let p = prev:
            if not same(self[p], self[i]):
                yield start!..=p
                start = i
        else:
            start = i
        prev = i
    if let s = start, let p = prev:
        yield s..=p

/// Maximal contiguous `Range<D>` runs of equal elements (or equal `by:` keys). Zero allocations, O(n).
pub fn run_ranges<D, T, K>(self: [D]T, by: T -> K = x => x) -> yields Range<D>:
    var start: ?D = none
    var prev: ?D = none
    var prev_key: ?K = none
    for i in D:
        let k = by(self[i])
        if let p = prev:
            if k != prev_key!:
                yield start!..=p
                start = i
                prev_key = k
        else:
            start = i
            prev_key = k
        prev = i
    if let s = start, let p = prev:
        yield s..=p

/// Group indices of `D` by their key in `K`, preserving order within each bucket. O(|D| + |K|).
pub fn buckets<D, K>(key: [D]K) -> [K]List<D>:
    var out: [K]List<D> = []
    for i in D: out[key[i]].push(i)
    out

/// For each value of `xs`, in order, the fold with `m` of all the others: what is left when
/// one is taken out, with no division or inverse needed. The fold keeps the order (the ones
/// before, then the ones after), so `m` need not commute. Works on any `Indexed`. O(n) calls of `m`.
///
///     for (x, rest) in [2, 3, 5].excl_fold(*): print(x, rest)    // 2 15, 3 10, 5 6
pub fn excl_fold<S: Indexed, T, M>(self: S, m: M) -> yields (x: T, rest: T):
    let k = Monoid<T>.of(m)
    var after = List.filled(self.len + 1, k.id)
    for i in (0..self.len).rev: after[i] = k.op(self.at(i), after[i + 1])
    var before = k.id
    for i in 0..self.len:
        yield (x: self.at(i), rest: k.op(before, after[i + 1]))
        before = k.op(before, self.at(i))

/// The distinct values of a coordinate compression, in increasing order, indexed by their
/// own domain `D`. `xs[i]` is the value at index `i`, `xs.at(val)` the index of a value that
/// was compressed, and `xs.first_ge(val)` / `xs.last_le(val)` the nearest index (`?D`) for any value.
pub struct Compressed<D, T>(vals: [D]T):
    /// The value at index `i` (or a `List<T>` slice when given a `Range<D>`). O(1) / O(r.len).
    pub fn get<K>(self, i: K) = vals[i]

    /// The compressed values over a range `r` of `xs.D`, as a list. O(r.len).
    pub fn slice(self, r: Range<D>) -> List<T> = [vals[i] for i in r]

    /// How many distinct values there are.
    pub @get fn len(self) -> Int = D.K

    /// The index of `val`, which must be one of the compressed values. O(log n).
    pub fn at(self, val: T) -> D:
        let i = vals.first_ge(val)
        assert i != none and vals[i!] == val  // `at` is for a value that was compressed
        i!

    /// The index of `val` if it was compressed, or `none`. O(log n).
    pub fn try_at(self, val: T) -> ?D:
        if let i = vals.first_ge(val), vals[i] == val: i else: none

    /// Whether `val` is one of the compressed values. O(log n).
    pub fn contains(self, val: T) -> Bool = self.try_at(val) != none

    /// The first index whose value is `>= val`, or `none`. O(log n).
    pub fn first_ge(self, val: T) -> ?D = vals.first_ge(val)

    /// The first index whose value is `> val`, or `none`. O(log n).
    pub fn first_gt(self, val: T) -> ?D = vals.first_gt(val)

    /// The last index whose value is `<= val`, or `none`. O(log n).
    pub fn last_le(self, val: T) -> ?D = vals.last_le(val)

    /// The last index whose value is `< val`, or `none`. O(log n).
    pub fn last_lt(self, val: T) -> ?D = vals.last_lt(val)

/// Coordinate compression: `let xs = compress(all)` sorts the values, drops the duplicates
/// and gives them a fresh domain `xs.D`, so an array over them is `[xs.D]Int` and an index of
/// one compression cannot be used in another. Works on any `Seq`. O(n log n).
pub fn compress<S: Seq, T>(all: S):
    var sorted_vals: List<T> = [x for x in all]
    sorted_vals.sort()
    dedup(mut sorted_vals)
    domain D = 0..sorted_vals.len
    Compressed<D, T>([D] i => sorted_vals[i.to_idx])

// ---- Range boundary cuts, keyed sorted lookup, ordered Map 4-lookups, and List/Array helpers

/// Left boundary cut of a range (valid even when empty).
pub @get fn l<D>(self: Range<D>) -> Bounds<D> = self.start

/// Right boundary cut of a range (valid even when empty).
pub @get fn r<D>(self: Range<D>) -> Bounds<D> = self.end

/// The two ends of a range as offsets, `(lo, hi)` with `lo <= hi`: what a range query walks.
/// A range written backwards, `P(3)..P(1)`, stops the program here: no query silently answers
/// as if it were empty. `l..l` is in order and empty.
pub @get fn ends<D>(self: Range<D>) -> (Int, Int):
    assert self.start.ord <= self.end.ord, "the range runs backwards: its end is before its start"
    (self.start.ord, self.end.ord)

/// First index in sorted `self` with `by(self[i]) > target`, or `none`. O(log n).
pub fn first_gt<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> ?D:
    let i = bsearch_first(0..D.count, i => by(self[D.nth(i)]) > target) ?? return none
    D.nth(i)

/// Last index in sorted `self` with `by(self[i]) < target`, or `none`. O(log n).
pub fn last_lt<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> ?D:
    let i = bsearch_last(0..D.count, i => by(self[D.nth(i)]) < target) ?? return none
    D.nth(i)

// ---- Permutations, monotone stack, and sliding-window extrema

/// Inverse of bijection `self: [I]V`: `inv[self[i]] == i` for all `i in I`.
/// Traps if `self` is not a bijection. O(|I|).
pub fn inverse<I, V>(self: [I]V) -> [V]I:
    assert I.count == V.count, "inverse requires domains of equal size"
    var inv: [V]?I = none
    for i in I:
        let v = self[i]
        assert inv[v] == none, "inverse requires a bijection (duplicate image)"
        inv[v] = i
    [V] v => inv[v]!

/// Function composition: `compose(f, g)[a] == g[f[a]]`. O(|A|).
pub fn compose<A, B, C>(f: [A]B, g: [B]C) -> [A]C = [A] a => g[f[a]]

/// Disjoint cycle decomposition of permutation `p`, each cycle starting at its smallest element. O(|V|).
pub fn perm_cycles<V>(p: [V]V) -> List<List<V>>:
    var vis: [V]Bool = false
    var out: List<List<V>> = []
    for start in V:
        if vis[start]: continue
        var cyc: List<V> = []
        var u = start
        while not vis[u]:
            vis[u] = true
            cyc.push(u)
            u = p[u]
        out.push(cyc)
    out

/// `k`-th iterate of permutation `p` (`k >= 0`), computed via cycle decomposition. O(|V|).
pub fn perm_pow<V>(p: [V]V, k: Int) -> [V]V:
    assert k >= 0, "perm_pow requires k >= 0"
    var out = [V] v => v
    for cyc in perm_cycles(p):
        let m = cyc.len
        let shift = k % m
        for j in 0..m:
            out[cyc[j]] = cyc[(j + shift) % m]
    out

/// Number of inversions: pairs `(i, j)` with `i < j` and `a[i] > a[j]`. Works on any `Seq`. O(n log n).
pub fn inversions<S: Seq, T>(a: S) -> Int:
    var cur: List<T> = [x for x in a]
    let n = cur.len
    if n <= 1: return 0
    var buf = List.filled(n, cur[0])
    var ans = 0
    var width = 1
    var to_buf = true
    while width < n:
        var lo = 0
        while lo < n:
            let mid = min(lo + width, n)
            let hi = min(lo + 2 * width, n)
            var (i, j, k) = (lo, mid, lo)
            if to_buf:
                while i < mid and j < hi:
                    if cur[j] < cur[i]:
                        buf[k] = cur[j]
                        ans += mid - i
                        j += 1
                    else:
                        buf[k] = cur[i]
                        i += 1
                    k += 1
                while i < mid:
                    buf[k] = cur[i]
                    i += 1
                    k += 1
                while j < hi:
                    buf[k] = cur[j]
                    j += 1
                    k += 1
            else:
                while i < mid and j < hi:
                    if buf[j] < buf[i]:
                        cur[k] = buf[j]
                        ans += mid - i
                        j += 1
                    else:
                        cur[k] = buf[i]
                        i += 1
                    k += 1
                while i < mid:
                    cur[k] = buf[i]
                    i += 1
                    k += 1
                while j < hi:
                    cur[k] = buf[j]
                    j += 1
                    k += 1
            lo = hi
        to_buf = not to_buf
        width *= 2
    ans

/// Nearest earlier index `j < i` with `self[j] < self[i]` (or `<=` when `strict: false`), or `none`. O(n).
pub fn prev_less<D, T>(self: [D]T, strict: Bool = true) -> [D]?D:
    var out: [D]?D = none
    var stk: List<D> = []
    for i in D:
        while not stk.is_empty and (
            if strict: self[stk.last!] >= self[i] else: self[stk.last!] > self[i]
        ):
            stk.pop()
        if not stk.is_empty: out[i] = stk.last!
        stk.push(i)
    out

/// Nearest later index `j > i` with `self[j] < self[i]` (or `<=` when `strict: false`), or `none`. O(n).
pub fn next_less<D, T>(self: [D]T, strict: Bool = true) -> [D]?D:
    var out: [D]?D = none
    var stk: List<D> = []
    for i in D.rev:
        while not stk.is_empty and (
            if strict: self[stk.last!] >= self[i] else: self[stk.last!] > self[i]
        ):
            stk.pop()
        if not stk.is_empty: out[i] = stk.last!
        stk.push(i)
    out

/// Nearest earlier index `j < i` with `self[j] > self[i]` (or `>=` when `strict: false`), or `none`. O(n).
pub fn prev_greater<D, T>(self: [D]T, strict: Bool = true) -> [D]?D:
    var out: [D]?D = none
    var stk: List<D> = []
    for i in D:
        while not stk.is_empty and (
            if strict: self[stk.last!] <= self[i] else: self[stk.last!] < self[i]
        ):
            stk.pop()
        if not stk.is_empty: out[i] = stk.last!
        stk.push(i)
    out

/// Nearest later index `j > i` with `self[j] > self[i]` (or `>=` when `strict: false`), or `none`. O(n).
pub fn next_greater<D, T>(self: [D]T, strict: Bool = true) -> [D]?D:
    var out: [D]?D = none
    var stk: List<D> = []
    for i in D.rev:
        while not stk.is_empty and (
            if strict: self[stk.last!] <= self[i] else: self[stk.last!] < self[i]
        ):
            stk.pop()
        if not stk.is_empty: out[i] = stk.last!
        stk.push(i)
    out

/// Minimum of every length-`k` contiguous window of `self` (`n - k + 1` values, empty if `k > n`). Works on any `Seq`. O(n).
pub fn sliding_min<S: Seq, T>(self: S, k: Int) -> List<T>:
    assert k >= 1, "sliding window size must be at least 1"
    var out: List<T> = []
    var dq: List<(pos: Int, val: T)> = []
    var head = 0
    var idx = 0
    for x in self:
        while head < dq.len and dq.last!.val >= x: dq.pop()
        dq.push((pos: idx, val: x))
        if dq[head].pos <= idx - k: head += 1
        if idx >= k - 1: out.push(dq[head].val)
        idx += 1
    out

/// Maximum of every length-`k` contiguous window of `self` (`n - k + 1` values, empty if `k > n`). Works on any `Seq`. O(n).
pub fn sliding_max<S: Seq, T>(self: S, k: Int) -> List<T>:
    assert k >= 1, "sliding window size must be at least 1"
    var out: List<T> = []
    var dq: List<(pos: Int, val: T)> = []
    var head = 0
    var idx = 0
    for x in self:
        while head < dq.len and dq.last!.val <= x: dq.pop()
        dq.push((pos: idx, val: x))
        if dq[head].pos <= idx - k: head += 1
        if idx >= k - 1: out.push(dq[head].val)
        idx += 1
    out

/// Swap two positions of a domain array in place; safely a no-op when `i == j`.
pub fn swap_at<D, T>(mut self: [D]T, i: D, j: D):
    if i == j: return
    let t = self[i]
    self[i] = self[j]
    self[j] = t

/// `self` cut or padded with `fill` to length `n`: `a.resized(size, Z(0))` instead of
/// `[if i < a.len: a[i] else: Z(0) for i in 0..size]`. Works on any `Indexed` sequence. O(n).
pub fn resized<S: Indexed, T>(self: S, n: Int, fill: T) -> List<T> = [
    if i < self.len: self.at(i) else: fill for i in 0..n
]

/// Move all elements out of `self` in O(1), leaving it empty.
pub fn take<T>(mut self: List<T>) -> List<T>:
    var out: List<T> = []
    swap(mut self, mut out)
    out

/// Insert `x` at index `i` (`0 <= i <= self.len`), shifting elements `i..` right by one. O(n).
pub fn insert<T>(mut self: List<T>, i: Int, x: T):
    assert 0 <= i and i <= self.len
    self.push(x)
    var j = self.len - 1
    while j > i:
        self.swap(j - 1, j)
        j -= 1

/// O(n). Remove and return the element at index `i` (`0 <= i < self.len`), shifting `i + 1..` left.
pub fn remove<T>(mut self: List<T>, i: Int) -> T:
    assert 0 <= i and i < self.len
    for j in i..self.len - 1:
        self.swap(j, j + 1)
    self.pop()!

// =============================================================================
// Spec surface (`= todo` where not yet implemented)
// =============================================================================

// --- Lazy views, slicing, and stream / combinator generators (W4, W6) ---

/// O(1) creation. Borrowed lazy or slice view over elements of type `T` (W4).
pub struct View<T>():
    /// O(1). Length of the view.
    @get pub fn len(self) -> Int = todo
    /// O(1). Whether the view is empty.
    @get pub fn is_empty(self) -> Bool = todo
    /// O(1). Element at 0-based offset `i`.
    pub fn at(self, i: Int) -> T = todo
    /// O(1) per step. Iterate elements of the view without allocating.
    pub fn iter(self) -> yields T = todo

/// O(1). Wrap any `Seq` as a lazy view so chained `.map`/`.filter` fuse without allocating.
pub fn view<S: Seq, T>(self: S) -> View<T> = todo

/// O(1) per step. Strided range `r.first, r.first + k, ...` within `r`; traps if `k <= 0`.
pub fn step_by<I>(self: Range<I>, k: Int) -> yields I = todo

/// O(1). Borrowed view skipping the first `k` elements (clamped to `self.len`; traps if `k < 0`).
pub fn drop_first<S: Indexed, T>(self: S, k: Int = 1) -> View<T> = todo

/// O(1). Borrowed view skipping the last `k` elements (clamped to `self.len`; traps if `k < 0`).
pub fn drop_last<S: Indexed, T>(self: S, k: Int = 1) -> View<T> = todo

/// O(1). Borrowed view of the first `min(k, self.len)` elements (traps if `k < 0`).
pub fn take_first<S: Indexed, T>(self: S, k: Int) -> View<T> = todo

/// O(1). Borrowed view of the last `min(k, self.len)` elements (traps if `k < 0`).
pub fn take_last<S: Indexed, T>(self: S, k: Int) -> View<T> = todo

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
pub fn group_by<S: Seq, T, K>(self: S, by: T -> K) -> HashMap<K, List<T>> = todo

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
pub fn span<D>(lo: Bounds<D>, hi: Bounds<D>) -> Range<D> = todo

// --- Sorted lookup, cut partition point, sorted set ops, and linear-time sorts ---

/// O(log n). Cut between the false prefix and true suffix of monotone `ok` on `self` (total; never `none`).
pub fn partition_point<D, T>(self: [D]T, ok: T -> Bool) -> Bounds<D> = todo

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

/// O(n log n), O(n) memory. Permutation of `D` sorting `self` stably by `by:` (ascending, or descending when `desc: true`).
pub fn argsort<D, T, K>(self: [D]T, by: T -> K = x => x, desc: Bool = false) -> List<D> = todo

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
fn scan_folds_every_prefix():
    assert scan([3, 1, 4, 1, 5], +) == [3, 4, 8, 9, 14]
    assert scan([3, 1, 4, 1, 5], max) == [3, 3, 4, 4, 5]
    let none_yet: List<Int> = []
    assert scan(none_yet, +).len == 0
    for _ in 0..50:
        let xs = [rand(-9, 9) for _ in 0..rand(1, 12)]
        let sums = scan(xs, +)
        for i in 0..xs.len: assert sums[i] == sum(xs[j] for j in 0..=i)
        var inp = xs.copy()
        inp.scan_in_place((a, b) => a + b)
        assert inp == sums
        var suf = xs.copy()
        suf.suffix_scan_in_place((a, b) => a + b)
        assert suf == suffix_scan(xs, +)
    var empty: List<Int> = []
    empty.scan_in_place((a, b) => a + b)
    empty.suffix_scan_in_place((a, b) => a + b)
    assert empty.is_empty

@test
fn sorted_lookup_and_linear_search():
    // Sequence first_ge / last_le return indices (?Int on List, ?D on [D]T)
    let xs = [1, 3, 3, 7, 10]
    assert xs.first_ge(3) == 1 and xs.first_ge(4) == 3 and xs.first_ge(11) == none
    assert xs.last_le(3) == 2 and xs.last_le(0) == none and xs.last_le(99) == 4
    assert xs.find(7) == 3 and xs.find(8) == none
    domain V = 1..=5
    let a = [V] v => xs[v.to_idx]
    assert a.first_ge(4) == V(4) and a.last_le(3) == V(3) and a.find(10) == V(5)
    assert first(x for x in xs if x > 3) == 7 and first(x for x in xs if x > 10) == none
    let words = ["pear", "fig", "banana", "kiwi"]
    assert words.min(by: w => w.len) == "fig" and words.max(by: w => w.len) == "banana"
    let nothing: List<Str> = []
    assert nothing.min(by: w => w.len) == none
    domain D = 0..5
    assert [w.len for w in D.windows(3)] == [3, 3, 3] and D.windows(3)[1] == (D(1)..=D(3))

@test
fn reorder_and_permutations_in_place():
    var a = [1, 1, 2, 3, 3, 3, 1]
    dedup(mut a)
    assert a == [1, 2, 3, 1]
    rotate_left(mut a, 1)
    assert a == [2, 3, 1, 1]
    rotate_left(mut a, -1)
    assert a == [1, 2, 3, 1]
    // Negative k rotates right (Euclidean modulo)
    var rot = [1, 2, 3, 4, 5]
    rotate_left(mut rot, -2)
    assert rot == [4, 5, 1, 2, 3]
    rotate_left(mut rot, -7)
    assert rot == [2, 3, 4, 5, 1]
    reverse(mut a)
    assert a == [1, 3, 2, 1]
    // next_permutation and prev_permutation are exact mirrors
    var p = [1, 2, 2]
    var fwd = [p.copy()]
    while next_permutation(mut p): fwd.push(p.copy())
    assert fwd == [[1, 2, 2], [2, 1, 2], [2, 2, 1]] and p == [1, 2, 2]
    var q = [2, 2, 1]
    var bwd = [q.copy()]
    while prev_permutation(mut q): bwd.push(q.copy())
    assert bwd == [[2, 2, 1], [2, 1, 2], [1, 2, 2]] and q == [2, 2, 1]
    // 4-element full round-trip forward and backward
    var perm = [1, 2, 3, 4]
    var up = [perm.copy()]
    while next_permutation(mut perm): up.push(perm.copy())
    assert up.len == 24 and perm == [1, 2, 3, 4]
    var perm_down = [4, 3, 2, 1]
    var down = [perm_down.copy()]
    while prev_permutation(mut perm_down): down.push(perm_down.copy())
    assert down.len == 24 and perm_down == [4, 3, 2, 1]
    assert down == up.rev
    // Edge cases: empty and singleton
    var empty: List<Int> = []
    assert not next_permutation(mut empty) and not prev_permutation(mut empty)
    var one = [42]
    assert not next_permutation(mut one) and not prev_permutation(mut one) and one == [42]
    assert runs("aabccc") == [('a', 2), ('b', 1), ('c', 3)]

@test
fn compress_gives_the_values_their_own_domain():
    let ys = [40, -7, 40, 1_000_000_000, 3]
    let xs = compress(ys)
    assert xs.len == 4 and xs.D.K == 4
    assert [xs[i] for i in xs.D] == [-7, 3, 40, 1_000_000_000]
    var cnt: [xs.D]Int = 0
    for y in ys: cnt[xs.at(y)] += 1
    assert [cnt[i] for i in xs.D] == [1, 1, 2, 1]
    assert xs.first_ge(4) == xs.at(40) and xs.first_ge(2_000_000_000) == none
    assert xs.last_le(4) == xs.at(3) and xs.last_le(-100) == none
    let other = compress([5, 6])
    assert other.D.K == 2 and xs.D.K == 4
    // Compressed range slicing
    let blk = xs.D.first..=xs.D.nth(2)
    assert xs.slice(blk) == [-7, 3, 40]
    assert xs[blk] == [-7, 3, 40]
    assert xs.slice(xs.D.nth(1)..xs.D.nth(1)) == []

@test
fn min_max_by_tie_breaking():
    // First extremum always wins ties; empty input always returns none
    let items = [
        (10, "first_min"),
        (5, "a"),
        (5, "b"),
        (20, "first_max"),
        (20, "second_max"),
        (5, "c"),
    ]
    assert items.min(by: p => p.0) == (5, "a")
    assert items.max(by: p => p.0) == (20, "first_max")
    assert [("x", 1), ("y", 1), ("z", 1)].min(by: p => p.1) == ("x", 1)
    assert [("x", 1), ("y", 1), ("z", 1)].max(by: p => p.1) == ("x", 1)
    let empty: List<(Int, Str)> = []
    assert empty.min(by: p => p.0) == none and empty.max(by: p => p.0) == none
    assert [x for x in items if x.0 < 0].min(by: p => p.0) == none
    assert [x for x in items if x.0 < 0].max(by: p => p.0) == none
    // Variadic min / max
    assert min(3, 1, 2) == 1 and max(3, 1, 2) == 3

@test
fn range_bounds_and_keyed_sorted_lookup():
    domain D = 1..=10
    // Range.l and Range.r boundary cuts even when empty
    let empty = D(4)..D(4)
    assert empty.l == D(4).l and empty.r == D(4).l and empty.l == empty.r
    let seg = D(2)..=D(5)
    assert seg.l == D(2).l and seg.r == D(5).r
    // first_ge / last_le with by: return ?Int
    let pairs = [(1, "a"), (3, "b"), (3, "c"), (7, "d")]
    assert pairs.first_ge(3, by: p => p.0) == 1
    assert pairs.first_ge(4, by: p => p.0) == 3
    assert pairs.first_ge(9, by: p => p.0) == none
    assert pairs.last_le(3, by: p => p.0) == 2
    assert pairs.last_le(0, by: p => p.0) == none

@test
fn list_insert_remove_take_and_swap_at():
    var xs: List<Int> = []
    xs.insert(0, 20)
    xs.insert(0, 10)
    xs.insert(2, 40)
    xs.insert(2, 30)
    assert xs == [10, 20, 30, 40]
    assert xs.remove(1) == 20 and xs == [10, 30, 40]
    assert xs.remove(0) == 10 and xs == [30, 40]
    assert xs.remove(1) == 40 and xs == [30]
    assert xs.remove(0) == 30 and xs.is_empty
    // Random insert/remove fuzz against slice reconstruction
    var cur: List<Int> = []
    var ref_xs: List<Int> = []
    for _ in 0..80:
        if cur.is_empty or rand(0, 1) == 0:
            let pos = rand(0, cur.len)
            let v = rand(-50, 50)
            cur.insert(pos, v)
            ref_xs = ref_xs[0..pos] + [v] + ref_xs[pos..ref_xs.len]
        else:
            let pos = rand(0, cur.len - 1)
            let got = cur.remove(pos)
            assert got == ref_xs[pos]
            ref_xs = ref_xs[0..pos] + ref_xs[pos + 1..ref_xs.len]
        assert cur == ref_xs
    // take moves out in O(1) leaving self empty (both built-in method and free fn)
    var u = [1, 2, 3, 4]
    let moved = u.take()
    assert u.is_empty and moved == [1, 2, 3, 4]
    var u2 = [5, 6, 7]
    let moved2 = take(mut u2)
    assert u2.is_empty and moved2 == [5, 6, 7]
    // swap_at is self-swap safe
    domain I = 0..4
    var a = [I] i => i.to_idx * 10
    a.swap_at(I(1), I(3))
    a.swap_at(I(2), I(2))
    assert [a[i] for i in I] == [0, 30, 20, 10]
    // resized pads with the fill or cuts, and leaves the original alone
    let p = [1, 2, 3]
    assert p.resized(5, 0) == [1, 2, 3, 0, 0] and p.resized(2, 0) == [1, 2]
    assert p.resized(0, 9).is_empty and p == [1, 2, 3]

@test
fn map_entry_remove_and_four_lookups():
    // Ordered Map 4-lookups return ?(K, V)
    var m = Map<Int, Str>()
    m[10] = "ten"
    m[20] = "twenty"
    m[30] = "thirty"
    assert m.first_ge(20) == (20, "twenty") and m.first_ge(21) == (
        30,
        "thirty",
    ) and m.first_ge(31) == none
    assert m.first_gt(20) == (30, "thirty") and m.first_gt(10) == (
        20,
        "twenty",
    ) and m.first_gt(30) == none
    assert m.last_le(20) == (20, "twenty") and m.last_le(19) == (10, "ten") and m.last_le(5) == none
    assert m.last_lt(20) == (10, "ten") and m.last_lt(30) == (
        20,
        "twenty",
    ) and m.last_lt(10) == none
    // Map.remove returns ?V (the removed value or none)
    assert m.remove(20) == "twenty"
    assert m.remove(20) == none
    assert m.get(20) == none and m.len == 2
    // Map compound assignment auto-inserts from operator identity; plain assign inserts/overwrites
    var cnt = Map<Int, Int>()
    cnt[5] += 3
    cnt[5] += 2
    cnt[8] min= 4
    cnt[8] min= 9
    cnt[9] max= -7
    assert cnt[5] == 5 and cnt[8] == 4 and cnt[9] == -7
    assert cnt.remove(5) == 5 and cnt.remove(5) == none
    // Differential check against brute-force scans on random maps
    for _ in 0..25:
        var rm = Map<Int, Int>()
        var ks: List<Int> = []
        for _ in 0..rand(0, 12):
            let k = rand(-10, 10)
            let v = rand(0, 99)
            rm[k] = v
            if k not in ks: ks.push(k)
        sort(mut ks)
        for q in -12..=12:
            let want_ge = first((k, rm[k]) for k in ks if k >= q)
            let want_gt = first((k, rm[k]) for k in ks if k > q)
            let want_le = [(k, rm[k]) for k in ks if k <= q].max(by: p => p.0)
            let want_lt = [(k, rm[k]) for k in ks if k < q].max(by: p => p.0)
            assert rm.first_ge(q) == want_ge
            assert rm.first_gt(q) == want_gt
            assert rm.last_le(q) == want_le
            assert rm.last_lt(q) == want_lt

@test
fn deduped_and_partition_leave_the_list_alone():
    let xs = [3, 1, 3, 2, 1]
    assert sorted(xs).deduped() == [1, 2, 3]
    assert xs.deduped() == xs
    let (small, big) = xs.partition(x => x < 3)
    assert small == [1, 2, 1] and big == [3, 3]
    assert xs == [3, 1, 3, 2, 1]
    for _ in 0..50:
        let ys = [rand(0, 5) for _ in 0..rand(0, 12)]
        let (even, odd) = ys.partition(y => y % 2 == 0)
        assert even.len + odd.len == ys.len and all(y % 2 == 0 for y in even) and all(y % 2 == 1 for y in odd)

@test
fn get_or_insert_stores_only_what_is_missing():
    var ids = Map<Str, Int>()
    assert ids.get_or_insert("a", ids.len) == 0 and ids.get_or_insert("b", ids.len) == 1
    assert ids.get_or_insert("a", 99) == 0 and ids.len == 2

@test
fn get_or_insert_with_makes_a_value_only_when_missing():
    var ids = Map<Str, Int>()
    var made = 0
    fn fresh() -> Int:
        made += 1
        made * 10
    assert ids.get_or_insert_with("a", fresh) == 10 and ids.get_or_insert_with("a", fresh) == 10
    assert ids.get_or_insert_with("b", () => 7) == 7 and made == 1

@test
fn update_starts_from_init_and_then_from_the_value():
    var best = Map<Str, Int>()
    best.update("a", 0, c => c + 1)
    best.update("a", 0, c => c + 1)
    best.update("b", 10, c => min(c, 4))
    assert best["a"] == 2 and best["b"] == 4 and best.len == 2

@test
fn runs_with_by_groups_neighbours_with_the_same_key():
    let rs = ["ab", "ac", "b", "ad"].runs(by: s => s[0])
    assert rs == [('a', 2), ('b', 1), ('a', 1)]
    let none_yet: List<Int> = []
    assert none_yet.runs(by: x => x).len == 0

@test
fn excl_fold_folds_all_but_one():
    assert [(x, rest) for (x, rest) in [2, 3, 5].excl_fold(*)] == [(2, 15), (3, 10), (5, 6)]
    assert [
        rest for (_, rest) in ["a", "b", "c"].excl_fold(Monoid("", (p, q) => p + q))
    ] == ["bc", "ac", "ab"]
    assert [rest for (_, rest) in [7].excl_fold(+)] == [0]

@test
fn windows_and_zip_walk_in_step():
    assert [w for w in [1, 2, 3, 4].windows(2)] == [[1, 2], [2, 3], [3, 4]]
    assert [w for w in [1, 2].windows(2)] == [[1, 2]] and [w for w in [1, 2].windows(3)].len == 0
    assert zip([1, 2, 3], ["a", "b"]) == [(1, "a"), (2, "b")] and zip([1], List<Int>()).len == 0

@test
fn pairs_and_with_prev_yield_neighbours():
    assert [p for p in [10, 20, 30].pairs()] == [(10, 20), (20, 30)]
    assert [p for p in [7].pairs()].len == 0
    let empty: List<Int> = []
    assert [p for p in empty.pairs()].len == 0
    assert [(p ?? -1, x) for (p, x) in [10, 20, 30].with_prev()] == [(-1, 10), (10, 20), (20, 30)]
    assert [(p ?? -1, x) for (p, x) in [7].with_prev()] == [(-1, 7)]
    assert [p for p in empty.with_prev()].len == 0

@test
fn cyclic_pairs_close_the_ring():
    assert [p for p in [1, 2, 3].cyclic_pairs()] == [(1, 2), (2, 3), (3, 1)]
    assert [p for p in [7].cyclic_pairs()] == [(7, 7)]
    let empty: List<Int> = []
    assert [p for p in empty.cyclic_pairs()].len == 0

/// The index of the first largest value (optionally under key `by:`), or `none` when empty:
/// `[3, 9, 2, 9].argmax` is `some(1)` and `words.argmax(by: w => w.len)` is the index of the
/// longest word. It is an index of `self`, so `a[a.argmax!]` is the largest value of a
/// `List<T>`, a `[1..=n]T` or a `[V]T` alike. O(n).
pub @get fn argmax<D, T, K>(self: [D]T, by: T -> K = x => x) -> ?D:
    var best: ?D = none
    var high: ?K = none
    for i in D:
        let k = by(self[i])
        if high == none or high! < k:
            best = i
            high = k
    best

/// The index of the first smallest value (optionally under key `by:`), or `none` when empty. O(n).
pub @get fn argmin<D, T, K>(self: [D]T, by: T -> K = x => x) -> ?D:
    var best: ?D = none
    var low: ?K = none
    for i in D:
        let k = by(self[i])
        if low == none or k < low!:
            best = i
            low = k
    best

@test
fn argmax_and_argmin_give_the_first_best_index():
    assert [3, 9, 2, 9].argmax == some(1)
    assert [3, 2, 9, 2].argmin == some(1)
    let empty: List<Int> = []
    assert empty.argmax == none and empty.argmin == none
    for _ in 0..50:
        let a = [rand(0, 5) for _ in 0..rand(1, 8)]
        let hi = a.argmax ?? -1
        assert all(a[i] < a[hi] for i in 0..hi) and all(a[i] <= a[hi] for i in 0..a.len)
        let lo = a.argmin ?? -1
        assert all(a[i] > a[lo] for i in 0..lo) and all(a[i] >= a[lo] for i in 0..a.len)
    let words = ["pear", "fig", "banana", "kiwi", "orange"]
    assert words.argmax(by: w => w.len) == some(2)
    assert words.argmin(by: w => w.len) == some(1)
    let no_words: List<Str> = []
    assert no_words.argmax(by: w => w.len) == none and no_words.argmin(by: w => w.len) == none
    domain W = 1..=3
    let ws = [W] i => words[i.to_idx]
    assert ws.argmax(by: w => w.len) == W(3) and ws.argmin(by: w => w.len) == W(2)

@test
fn prod_and_is_sorted_cover_sequences():
    assert prod([2, 3, 5]) == 30
    let empty_int: List<Int> = []
    assert prod(empty_int) == 1
    type Z = Mod<998_244_353>
    assert prod([Z(2), Z(3), Z(7)]) == Z(42)
    let empty_z: List<Z> = []
    assert prod(empty_z) == Z(1)
    assert prod(x + 1 for x in 1..=4) == 120
    assert [1, 2, 2, 5].is_sorted() and not [1, 3, 2].is_sorted()
    assert empty_int.is_sorted() and [42].is_sorted()
    assert ["a", "bb", "ccc"].is_sorted(by: s => s.len)
    assert not ["bb", "a", "ccc"].is_sorted(by: s => s.len)
    assert "aabbz".is_sorted() and not "abaz".is_sorted()
    domain V = 1..=4
    let sorted_v = [V] i => i.to_idx * 2
    assert sorted_v.is_sorted()

@test
fn nth_element_merge_sorted_and_rotate_right():
    var sub = [0, 1, 2, 3, 4, 5]
    reverse(mut sub, range: 1..5)
    assert sub == [0, 4, 3, 2, 1, 5]
    rotate_left(mut sub, 1, range: 1..=4)
    assert sub == [0, 3, 2, 1, 4, 5]
    rotate_right(mut sub, 1, range: 1..=4)
    assert sub == [0, 4, 3, 2, 1, 5]
    for _ in 0..100:
        let n = rand(1, 20)
        let orig = [rand(-5, 5) for _ in 0..n]
        let k = rand(0, n - 1)
        var a = orig.copy()
        nth_element(mut a, k)
        let s = sorted(orig)
        assert a[k] == s[k]
        assert all(a[i] <= a[k] for i in 0..k)
        assert all(a[i] >= a[k] for i in k + 1..n)
        assert sorted(a) == s
    assert merge_sorted([1, 3, 5], [2, 3, 4, 6]) == [1, 2, 3, 3, 4, 5, 6]
    let empty: List<Int> = []
    assert merge_sorted(empty, [1, 2]) == [1, 2] and merge_sorted([1, 2], empty) == [1, 2]
    var r = [1, 2, 3, 4, 5]
    rotate_right(mut r, 2)
    assert r == [4, 5, 1, 2, 3]
    rotate_right(mut r, 7)
    assert r == [2, 3, 4, 5, 1]

@test
fn chunk_by_run_ranges_buckets_and_chunks():
    // Alternating runs on Str (CF 2069E pattern)
    let s = "ABABBAAB"
    let alt = [r.len for r in s.chunk_by((x, y) => x != y)]
    assert alt == [4, 2, 2]
    // Equal runs on Str and [D]T
    assert "aabccc".runs() == [('a', 2), ('b', 1), ('c', 3)]
    assert [r.len for r in "aabccc".run_ranges()] == [2, 1, 3]
    assert [p for p in "abc".pairs()] == [('a', 'b'), ('b', 'c')]
    assert [p for p in "ab".cyclic_pairs()] == [('a', 'b'), ('b', 'a')]
    let empty_str = ""
    assert [r for r in empty_str.chunk_by((x, y) => x == y)].is_empty
    assert [r for r in empty_str.run_ranges()].is_empty
    // Keyed run_ranges over a domain array
    domain V = 1..=5
    let vals = [V] i => [10, 12, 21, 23, 30][i.to_idx]
    let tens = [r for r in vals.run_ranges(by: x => x / 10)]
    assert tens == [V(1)..=V(2), V(3)..=V(4), V(5)..=V(5)]
    // Typed buckets
    domain K = 0..3
    let keys = [V] i => K([2, 0, 2, 1, 0][i.to_idx])
    let b = buckets(keys)
    assert b[K(0)] == [V(2), V(5)] and b[K(1)] == [V(4)] and b[K(2)] == [V(1), V(3)]
    // Chunks
    assert [c for c in [1, 2, 3, 4, 5].chunks(2)] == [[1, 2], [3, 4], [5]]
    let empty_l: List<Int> = []
    assert [c for c in empty_l.chunks(3)].is_empty

@test
fn suffix_scan_matches_right_to_left_fold():
    assert suffix_scan([3, 1, 4, 1, 5], +) == [14, 11, 10, 6, 5]
    assert suffix_scan([3, 1, 4, 1, 5], max) == [5, 5, 5, 5, 5]
    domain V = 1..=4
    let a = [V] i => [2, 3, 5, 7][i.to_idx]
    let sf = suffix_scan(a, *)
    assert sf[V(1)] == 210 and sf[V(2)] == 105 and sf[V(3)] == 35 and sf[V(4)] == 7
    let empty: List<Int> = []
    assert suffix_scan(empty, +).is_empty

@test
fn first_gt_and_last_lt_match_linear_scan():
    let xs = [1, 3, 3, 7, 10]
    assert xs.first_gt(3) == some(3) and xs.first_gt(10) == none and xs.first_gt(0) == some(0)
    assert xs.last_lt(3) == some(0) and xs.last_lt(1) == none and xs.last_lt(99) == some(4)
    let c = compress([10, 3, 7, 3, 1])
    assert c.contains(7) and not c.contains(5)
    assert c.try_at(7) == c.at(7) and c.try_at(5) == none
    assert c.first_gt(3) == c.at(7) and c.first_gt(10) == none
    assert c.last_lt(3) == c.at(1) and c.last_lt(1) == none
    // Keyed lookup and domain array lookup
    let pairs = [(1, "a"), (3, "b"), (3, "c"), (7, "d")]
    assert pairs.first_gt(3, by: p => p.0) == some(3)
    assert pairs.last_lt(3, by: p => p.0) == some(0)
    domain V = 1..=5
    let va = [V] i => xs[i.to_idx]
    assert va.first_gt(3) == V(4) and va.last_lt(3) == V(1)
    assert xs[1..4].first_gt(3) == some(2) and xs[1..4].last_lt(3) == none
    assert xs[0..4].last_lt(3) == some(0) and xs[0..4].first_gt(7) == none
    assert pairs[1..4].first_gt(3, by: p => p.0) == some(2)
    assert pairs[0..3].last_lt(3, by: p => p.0) == some(0)
    let s = "abccd"
    assert s[1..4].first_gt('b') == some(1) and s[1..4].last_lt('c') == some(0)
    for _ in 0..50:
        let a = sorted([rand(-5, 5) for _ in 0..rand(0, 12)])
        for q in -6..=6:
            assert a.first_gt(q) == first(i for i in 0..a.len if a[i] > q)
            assert a.last_lt(q) == max(i for i in 0..a.len if a[i] < q)

@test
fn permutations_and_inversions():
    domain I = 1..=5
    let p = [I] i => I([3, 5, 4, 1, 2][i.to_idx])
    let inv = p.inverse()
    for i in I:
        assert inv[p[i]] == i and p[inv[i]] == i
    domain P = 0..5
    let q = [P] i => I([3, 5, 4, 1, 2][i.to_idx])
    let inv_q = q.inverse()
    for i in P:
        assert inv_q[q[i]] == i and q[inv_q[q[i]]] == q[i]
    let id = compose(p, inv)
    for i in I: assert id[i] == i
    let cycs = perm_cycles(p)
    assert cycs == [[I(1), I(3), I(4)], [I(2), I(5)]]
    // perm_pow matches repeated compose and handles large k
    let p0 = perm_pow(p, 0)
    for i in I: assert p0[i] == i
    let p2 = compose(p, p)
    let p2_fast = perm_pow(p, 2)
    for i in I: assert p2[i] == p2_fast[i]
    let p6 = perm_pow(p, 6)
    for i in I: assert p6[i] == i
    let p_big = perm_pow(p, 1_000_000_000_004)
    for i in I: assert p_big[i] == p2[i]
    // Inversions against O(n^2) brute force
    assert inversions([1, 2, 3]) == 0 and inversions([3, 2, 1]) == 3
    let empty: List<Int> = []
    assert inversions(empty) == 0 and inversions([5]) == 0
    for _ in 0..60:
        let a = [rand(0, 6) for _ in 0..rand(0, 15)]
        var brute = 0
        for i in 0..a.len:
            for j in i + 1..a.len:
                if a[i] > a[j]: brute += 1
        assert inversions(a) == brute

@test
fn monotone_stack_and_sliding_extrema():
    domain V = 1..=5
    let a = [V] i => [3, 1, 4, 1, 5][i.to_idx]
    let pl = a.prev_less()
    let nl = a.next_less()
    assert pl[V(1)] == none and pl[V(3)] == V(2) and pl[V(4)] == none
    assert nl[V(1)] == V(2) and nl[V(3)] == V(4) and nl[V(5)] == none
    // Differential test against O(n^2) brute force on random arrays (strict & non-strict)
    for _ in 0..60:
        let n = rand(0, 12)
        let xs = [rand(0, 4) for _ in 0..n]
        let pl_s = xs.prev_less()
        let pl_ns = xs.prev_less(strict: false)
        let nl_s = xs.next_less()
        let nl_ns = xs.next_less(strict: false)
        let pg_s = xs.prev_greater()
        let pg_ns = xs.prev_greater(strict: false)
        let ng_s = xs.next_greater()
        let ng_ns = xs.next_greater(strict: false)
        for i in 0..n:
            assert pl_s[i] == max(j for j in 0..i if xs[j] < xs[i])
            assert pl_ns[i] == max(j for j in 0..i if xs[j] <= xs[i])
            assert nl_s[i] == first(j for j in i + 1..n if xs[j] < xs[i])
            assert nl_ns[i] == first(j for j in i + 1..n if xs[j] <= xs[i])
            assert pg_s[i] == max(j for j in 0..i if xs[j] > xs[i])
            assert pg_ns[i] == max(j for j in 0..i if xs[j] >= xs[i])
            assert ng_s[i] == first(j for j in i + 1..n if xs[j] > xs[i])
            assert ng_ns[i] == first(j for j in i + 1..n if xs[j] >= xs[i])
    // Sliding min/max on [V]T and randomized differential test
    assert a.sliding_min(3) == [1, 1, 1] and a.sliding_max(3) == [4, 4, 5]
    assert a.sliding_min(6).is_empty and a.sliding_max(6).is_empty
    for _ in 0..60:
        let n = rand(0, 15)
        let xs = [rand(-10, 10) for _ in 0..n]
        let k = rand(1, n + 2)
        let want_min = [min(xs[i..i + k])! for i in 0..max(0, n - k + 1)]
        let want_max = [max(xs[i..i + k])! for i in 0..max(0, n - k + 1)]
        assert xs.sliding_min(k) == want_min
        assert xs.sliding_max(k) == want_max

@test
fn seq_and_indexed_traits_unify_sequences_and_views():
    // cf2084h regression: Str.runs() directly without conversion
    let s: Str = "0011100"
    let blocks = [r.len for r in s.runs()]
    assert blocks == [2, 3, 2]
    assert s.first == '0' and s.last == '0' and s.last(2) == '1'
    assert s.get(2) == '1' and s.get(9) == none and s.at(2) == '1'
    assert "".first == none and "".last == none and "".get(0) == none
    assert s[1..5].runs().len == 2
    assert [r.len for r in s.rev.runs()] == [2, 3, 2]

    // List, domain-indexed [V]Int, Range, slice, and rev all implement Seq + Indexed
    let xs = [1, 1, 2, 2, 2]
    assert [r.len for r in xs.runs()] == [2, 3]
    assert xs.is_sorted() and not xs.rev.is_sorted()

    domain V = 1..=5
    let arr: [V]Int = [10, 10, 20, 30, 30]
    assert [r.len for r in arr.runs()] == [2, 1, 2]
    assert arr.is_sorted() and arr.at(2) == 20 and arr.get(2) == 20 and arr.get(5) == none

    let r = 1..=5
    assert r.runs().len == 5 and r.is_sorted()
    assert r.first == 1 and r.last == 5 and r.last(1) == 4
    assert r[0] == 1 and r[4] == 5 and r.at(2) == 3 and r.get(2) == 3 and r.get(5) == none
    assert sum(r) == 15 and min(r) == 1 and max(r) == 5 and prod(r) == 120
    assert [p for p in r.pairs()] == [(1, 2), (2, 3), (3, 4), (4, 5)]
    assert [e.rest for e in r.excl_fold(+)] == [14, 13, 12, 11, 10]

    // Generalized Seq/Indexed ops on views and arrays without copying
    let vs = [1, 3, 5, 2, 4, 6]
    assert merge_sorted(vs[0..3], vs[3..6]) == [1, 2, 3, 4, 5, 6]
    assert zip(vs[0..3], s[0..3]) == [(1, '0'), (3, '0'), (5, '1')]
    let cv = compress(vs[1..5])
    assert cv.len == 4 and cv.at(2).ord == 0 and cv.at(5).ord == 3
    assert inversions(vs[1..5]) == 3
    assert vs[1..4].resized(5, 0) == [3, 5, 2, 0, 0]
    assert vs[1..6].sliding_min(2) == [3, 2, 2, 4]
    assert vs[1..6].sliding_max(2) == [5, 5, 4, 6]

@test
fn distinct_with_by_key_keeps_first_seen():
    let pairs = [(1, "a"), (2, "b"), (1, "c"), (3, "d"), (2, "e")]
    assert pairs.distinct(by: p => p.0) == [(1, "a"), (2, "b"), (3, "d")]
    let words = ["ant", "ape", "bat", "arc", "box"]
    assert words.distinct(by: w => w[0]) == ["ant", "bat"]
    let empty: List<(Int, Str)> = []
    assert empty.distinct(by: p => p.0) == []
    domain D = 0..5
    let arr: [D]Int = [10, 21, 30, 41, 52]
    assert arr.distinct(by: x => x % 10) == [10, 21, 52]
    assert "aAbBcA".distinct(by: c => if c >= 'a': c - 'a' else: c - 'A') == ['a', 'b', 'c']

@test
fn scan_prod_and_rotate_method_syntax():
    let xs = [3, 1, 4, 1, 5]
    assert xs.scan(+) == [3, 4, 8, 9, 14]
    assert xs.scan(min) == [3, 1, 1, 1, 1]
    assert xs.suffix_scan(+) == [14, 11, 10, 6, 5]
    assert xs.suffix_scan(max) == [5, 5, 5, 5, 5]
    assert xs.rev.scan(max).rev == [5, 5, 5, 5, 5]
    assert xs.prod() == 60

    domain D = 0..4
    let a: [D]Int = [4, 2, 5, 1]
    let p = a.scan(min)
    let s = a.suffix_scan(max)
    assert [p[i] for i in D] == [4, 2, 2, 1]
    assert [s[i] for i in D] == [5, 5, 5, 1]
    assert a.prod() == 40

    assert xs.rotated_left(2) == [4, 1, 5, 3, 1]
    assert xs.rotated_left(-1) == [5, 3, 1, 4, 1]
    assert xs.rotated_right(2) == [1, 5, 3, 1, 4]
    assert a.rotated_left(1) == [2, 5, 1, 4]
    assert "abcd".rotated_left(1) == ['b', 'c', 'd', 'a']
    let empty: List<Int> = []
    assert empty.rotated_left(3) == []
    var m = [1, 2, 3, 4]
    m.rotate_left(1)
    assert m == [2, 3, 4, 1]
    m.rotate_right(2)
    assert m == [4, 1, 2, 3]

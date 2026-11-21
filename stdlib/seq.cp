//! Sequences: reductions, scans, runs, searching, permutations, monotone stacks, and compression.
//!
//! - Reductions & extrema: `prod`, `is_sorted`, `min`/`max` (with `by:`), `argmax`/`argmin` (with `by:`).
//! - Binary & unimodal search: `bsearch_first`, `bsearch_last`, `bsearch_real`, `ternary_max`/`ternary_min`, `ternary_max_real`/`ternary_min_real`, `parallel_bsearch`.
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

/// Every prefix folded with `f`: `[a0, f(a0, a1), f(f(a0, a1), a2), ...]`, as long as `xs`
/// and indexed as `xs` is: a `List<T>` gives a `List<T>`, a `[D]T` a `[D]T`.
/// `scan(xs, +)` is the running sum and `scan(xs, max)` the running maximum; no identity
/// is needed because the first value starts the fold. O(n) calls of `f`.
pub fn scan<D, T>(xs: [D]T, f: (T, T) -> T) -> [D]T:
    var out = xs
    var before: ?D = none
    for i in D:
        if let b = before: out[i] = f(out[b], out[i])
        before = i
    out

/// Every suffix folded right-to-left with `f`: `out[i] = f(xs[i], out[i + 1])`, as long as `xs`
/// and indexed as `xs` is. No identity is needed. O(n) calls of `f`.
pub fn suffix_scan<D, T>(xs: [D]T, f: (T, T) -> T) -> [D]T:
    var out = xs
    var after: ?D = none
    for i in D.rev:
        if let a = after: out[i] = f(out[i], out[a])
        after = i
    out

/// Fold every prefix of `self` in place with `f`: `self[i] = f(self[i - 1], self[i])`.
/// Zero allocations. O(n) calls of `f`.
pub fn scan_in_place<T>(mut self: List<T>, f: (T, T) -> T):
    var prev: ?T = none
    for i in 0..self.len:
        if let p = prev: self[i] = f(p, self[i])
        prev = self[i]

/// Fold every suffix of `self` right-to-left in place with `f`: `self[i] = f(self[i], self[i + 1])`.
/// Zero allocations. O(n) calls of `f`.
pub fn suffix_scan_in_place<T>(mut self: List<T>, f: (T, T) -> T):
    var next: ?T = none
    for i in (0..self.len).rev:
        if let n = next: self[i] = f(self[i], n)
        next = self[i]

/// Product of all elements of `it` (`T.one` when empty). Works on any `Seq` (`Str`, `List`, `[D]T`, `Range`, views) of `Int`, `Mod<P>`, `F64`. O(n).
pub fn prod<S: Seq, T>(it: S) -> T:
    var out = T.one
    for x in it: out *= x
    out

/// Whether `self` is non-decreasing (optionally under key `by:`). Works on any `Seq`. O(n).
pub fn is_sorted<S: Seq, T, K>(self: S, by: T -> K = x => x) -> Bool:
    var prev: ?K = none
    for x in self:
        let k = by(x)
        if let p = prev, k < p: return false
        prev = k
    true

/// The first position of `r` where `ok` holds, or `none`. `ok` must be false and then true
/// along the range; the two ends are checked against that. O(log n) calls of `ok`.
pub fn bsearch_first<I>(r: Range<I>, ok: I -> Bool) -> ?I:
    if r.is_empty: return none
    var lo = r.first!
    var hi = r.last!
    if not ok(hi): return none
    assert not (ok(lo) and not ok(hi))  // `ok` must not switch back to false
    while lo < hi:
        let mid = (lo + (hi - lo) / 2)!
        if ok(mid): hi = mid else: lo = (mid + 1)!
    lo

/// The last position of `r` where `ok` holds, or `none`. `ok` must be true and then false
/// along the range. O(log n) calls of `ok`.
pub fn bsearch_last<I>(r: Range<I>, ok: I -> Bool) -> ?I:
    if r.is_empty: return none
    var lo = r.first!
    var hi = r.last!
    if not ok(lo): return none
    while lo < hi:
        let span = hi - lo
        let mid = (lo + (span - span / 2))!
        if ok(mid): lo = mid else: hi = (mid - 1)!
    lo

/// Fixed-iteration binary search on `[lo, hi]` for monotone `ok` (false then true).
/// Returns the least `x` in `[lo, hi]` with `ok(x)` to within `(hi - lo) / 2^steps`, or `none`
/// when `not ok(hi)`. `steps + 1` calls of `ok`.
pub fn bsearch_real(lo: F64, hi: F64, ok: F64 -> Bool, steps: Int = 100) -> ?F64:
    if not ok(hi): return none
    var (l, r) = (lo, hi)
    for _ in 0..steps:
        let m = (l + r) * 0.5
        if ok(m): r = m else: l = m
    r

/// Position maximizing a discrete unimodal function `f` over `r`, or `none` if `r` is empty.
/// `f` must be strictly rising then falling (plateaus only at the optimum). O(log |r|) calls of `f`.
pub fn ternary_max<I, K>(r: Range<I>, f: I -> K) -> ?I:
    if r.is_empty: return none
    var lo = r.first!
    var hi = r.last!
    while lo < hi:
        let mid = (lo + (hi - lo) / 2)!
        if f(mid) < f((mid + 1)!): lo = (mid + 1)! else: hi = mid
    lo

/// Position minimizing a discrete unimodal function `f` over `r`, or `none` if `r` is empty.
/// `f` must be strictly falling then rising (plateaus only at the optimum). O(log |r|) calls of `f`.
pub fn ternary_min<I, K>(r: Range<I>, f: I -> K) -> ?I:
    if r.is_empty: return none
    var lo = r.first!
    var hi = r.last!
    while lo < hi:
        let mid = (lo + (hi - lo) / 2)!
        if f(mid) > f((mid + 1)!): lo = (mid + 1)! else: hi = mid
    lo

/// Point in `[lo, hi]` maximizing continuous unimodal `f` via golden-section search.
/// Within `(hi - lo) * 0.618^steps` of the optimum. `steps + 2` calls of `f`.
pub fn ternary_max_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64:
    let inv_phi = 0.6180339887498949
    var (l, r) = (lo, hi)
    var m1 = r - (r - l) * inv_phi
    var m2 = l + (r - l) * inv_phi
    var f1 = f(m1)
    var f2 = f(m2)
    for _ in 0..steps:
        if f1 < f2:
            l = m1
            m1 = m2
            f1 = f2
            m2 = l + (r - l) * inv_phi
            f2 = f(m2)
        else:
            r = m2
            m2 = m1
            f2 = f1
            m1 = r - (r - l) * inv_phi
            f1 = f(m1)
    (l + r) * 0.5

/// Point in `[lo, hi]` minimizing continuous unimodal `f` via golden-section search.
/// Within `(hi - lo) * 0.618^steps` of the optimum. `steps + 2` calls of `f`.
pub fn ternary_min_real(lo: F64, hi: F64, f: F64 -> F64, steps: Int = 100) -> F64:
    let inv_phi = 0.6180339887498949
    var (l, r) = (lo, hi)
    var m1 = r - (r - l) * inv_phi
    var m2 = l + (r - l) * inv_phi
    var f1 = f(m1)
    var f2 = f(m2)
    for _ in 0..steps:
        if f1 > f2:
            l = m1
            m1 = m2
            f1 = f2
            m2 = l + (r - l) * inv_phi
            f2 = f(m2)
        else:
            r = m2
            m2 = m1
            f2 = f1
            m1 = r - (r - l) * inv_phi
            f1 = f(m1)
    (l + r) * 0.5

/// Many binary searches over one timeline `r` at once. `reset()` empties the user's
/// structure, `apply(t)` adds the event at time `t`, and `ok(q)` asks query `q` of the
/// structure as it stands; `out[q]` is the first `t` with `ok(q)` once every event up to
/// `t` is applied, or `none`. Law: each `ok(q)` stays true once it holds. O(log n)
/// rounds, each one `reset`, `n` calls of `apply` and one `ok` per open query, for
/// n = `r.len`; memory O(n + queries).
pub fn parallel_bsearch<I>(
    queries: Int,
    r: Range<I>,
    reset: () -> (),
    apply: I -> (),
    ok: Int -> Bool,
) -> List<?I>:
    let n = r.len
    var lo = List.filled(queries, 0)
    var hi = List.filled(queries, n)
    while true:
        var start = List.filled(n + 1, 0)
        for q in 0..queries:
            if lo[q] < hi[q]: start[(lo[q] + hi[q]) / 2 + 1] += 1
        if all(lo[q] == hi[q] for q in 0..queries): break
        for t in 0..n: start[t + 1] += start[t]
        var order = List.filled(start[n], 0)
        var fill = [x for x in start]
        for q in 0..queries:
            if lo[q] < hi[q]:
                let t = (lo[q] + hi[q]) / 2
                order[fill[t]] = q
                fill[t] += 1
        reset()
        for t in 0..n:
            apply((r.first! + t)!)
            for at in start[t]..start[t + 1]:
                let q = order[at]
                if ok(q): hi[q] = t else: lo[q] = t + 1
    [if lo[q] == n: none else: (r.first! + lo[q])! for q in 0..queries]

/// Reverse `a` in place (or only the subrange `range` when given). O(n).
pub fn reverse<T>(mut a: List<T>, range: ?Range<Int> = none):
    var lo = 0
    var hi = a.len
    if let r = range:
        if r.is_empty: return
        lo = r.first!
        hi = r.last! + 1
    var (i, j) = (lo, hi - 1)
    while i < j:
        a.swap(i, j)
        i += 1
        j -= 1

/// Drop each value equal to the one before it, in place: a sorted list keeps one of each.
pub fn dedup<T>(mut a: List<T>):
    var kept = 0
    for i in 0..a.len:
        if kept == 0 or a[kept - 1] != a[i]:
            a[kept] = a[i]
            kept += 1
    a.truncate(kept)

/// `self` without each value equal to the one before it: the value form of `dedup`, as `sorted` is of `sort`.
/// Works on any `Seq`. O(n) time without hashing.
pub fn deduped<S: Seq, T>(self: S) -> List<T>:
    var out: List<T> = []
    for x in self:
        if out.is_empty or out.last! != x:
            out.push(x)
    out

/// Distinct values of `self` in first-seen order. Works on any `Seq`.
/// O(n) expected time, O(n) space via `HashSet`. Unlike `dedup` / `deduped`
/// (adjacent duplicates only, O(n) time without hashing), `distinct` drops
/// duplicates anywhere in the sequence.
pub fn distinct<S: Seq, T>(self: S) -> List<T>:
    var seen = HashSet<T>()
    var out: List<T> = []
    for x in self:
        if seen.add(x):
            out.push(x)
    out

/// The values of `self` that `keep` holds for, then the ones it does not, each in the order they came.
pub fn partition<S: Seq, T>(self: S, keep: T -> Bool) -> (List<T>, List<T>):
    var (yes, no): (List<T>, List<T>) = ([], [])
    for x in self:
        if keep(x): yes.push(x)
        else: no.push(x)
    (yes, no)

/// Partition `a` in place so `a[k]` holds the element that would sit at `k` in sorted order,
/// with `a[i] <= a[k]` for `i < k` and `a[i] >= a[k]` for `i > k`. O(n) expected, O(1) extra space.
pub fn nth_element<T>(mut a: List<T>, k: Int):
    assert 0 <= k and k < a.len, "nth_element index out of range"
    var (lo, hi) = (0, a.len - 1)
    var state = 1_234_567
    while lo < hi:
        state = (state * 48271) % 2_147_483_647
        let piv = a[lo + (state % (hi - lo + 1))]
        var (lt, i, gt) = (lo, lo, hi)
        while i <= gt:
            if a[i] < piv:
                a.swap(lt, i)
                lt += 1
                i += 1
            elif piv < a[i]:
                a.swap(i, gt)
                gt -= 1
            else:
                i += 1
        if k < lt: hi = lt - 1
        elif k > gt: lo = gt + 1
        else: return

/// Merge two sorted lists into one sorted list, taking from `a` before `b` on ties. O(|a| + |b|).
pub fn merge_sorted<T>(a: List<T>, b: List<T>) -> List<T>:
    var out: List<T> = []
    var (i, j) = (0, 0)
    while i < a.len and j < b.len:
        if b[j] < a[i]:
            out.push(b[j])
            j += 1
        else:
            out.push(a[i])
            i += 1
    while i < a.len:
        out.push(a[i])
        i += 1
    while j < b.len:
        out.push(b[j])
        j += 1
    out

/// Every run of `k` neighbours, left to right: `[1, 2, 3].windows(2)` gives `[1, 2]` then `[2, 3]`.
/// Works on any `Indexed` sequence (`List<T>`, `[D]T`, `Str`, `Range`, views). Nothing when `k > self.len`. O(k) per window.
pub fn windows<S: Indexed, T>(self: S, k: Int) -> yields List<T>:
    assert k >= 1, "a window holds at least one element"
    for i in 0..self.len - k + 1:
        var w: List<T> = []
        for j in i..i + k: w.push(self.at(j))
        yield w

/// Non-overlapping slices of length `k` (the last may be shorter). Works on any `Indexed` sequence. O(n) total.
pub fn chunks<S: Indexed, T>(self: S, k: Int) -> yields List<T>:
    assert k >= 1, "a chunk holds at least one element"
    var i = 0
    while i < self.len:
        let end = min(i + k, self.len)
        var c: List<T> = []
        for j in i..end: c.push(self.at(j))
        yield c
        i = end

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

/// The pairs `(a[i], b[i])`, as many as the shorter list has.
pub fn zip<A, B>(a: List<A>, b: List<B>) -> List<(A, B)> = [
    (a[i], b[i]) for i in 0..min(a.len, b.len)
]

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

/// Rotate `a` left by `k` in place (or only the subrange `range` when given):
/// what was at offset `k % len` comes first. Signed Euclidean `k`: negative `k`
/// rotates right. O(n), O(1) extra space.
pub fn rotate_left<T>(mut a: List<T>, k: Int, range: ?Range<Int> = none):
    var lo = 0
    var hi = a.len
    if let r = range:
        if r.is_empty: return
        lo = r.first!
        hi = r.last! + 1
    let len = hi - lo
    if len <= 1: return
    let s = k % len
    if s == 0: return
    reverse(mut a, range: lo..lo + s)
    reverse(mut a, range: lo + s..hi)
    reverse(mut a, range: lo..hi)

/// Rotate `a` right by `k` in place (or only the subrange `range` when given). O(n).
pub fn rotate_right<T>(mut a: List<T>, k: Int, range: ?Range<Int> = none):
    rotate_left(mut a, -k, range: range)

/// Step `a` to the next arrangement in sorted order, in place. Returns false, leaving `a`
/// sorted ascending, when it was the last one. O(n), amortized O(1).
pub fn next_permutation<T>(mut a: List<T>) -> Bool:
    var i = a.len - 2
    while i >= 0 and not (a[i] < a[i + 1]): i -= 1
    if i < 0:
        reverse(mut a)
        return false
    var j = a.len - 1
    while not (a[i] < a[j]): j -= 1
    a.swap(i, j)
    var (l, r) = (i + 1, a.len - 1)
    while l < r:
        a.swap(l, r)
        l += 1
        r -= 1
    true

/// Step `a` to the previous arrangement in sorted order, in place. Returns false, leaving `a`
/// sorted descending, when it was the first (ascending) one. O(n), amortized O(1).
pub fn prev_permutation<T>(mut a: List<T>) -> Bool:
    var i = a.len - 2
    while i >= 0 and not (a[i + 1] < a[i]): i -= 1
    if i < 0:
        reverse(mut a)
        return false
    var j = a.len - 1
    while not (a[j] < a[i]): j -= 1
    a.swap(i, j)
    var (l, r) = (i + 1, a.len - 1)
    while l < r:
        a.swap(l, r)
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
/// one compression cannot be used in another. O(n log n).
pub fn compress<T>(all: List<T>):
    var sorted_vals = sorted(all)
    dedup(mut sorted_vals)
    domain D = 0..sorted_vals.len
    Compressed<D, T>([D] i => sorted_vals[i.to_idx])

// ---- Range boundary cuts, keyed sorted lookup, ordered Map 4-lookups, and List/Array helpers

/// Left boundary cut of a range (valid even when empty).
pub @get fn l<D>(self: Range<D>) -> Bounds<D> = self.lo

/// Right boundary cut of a range (valid even when empty).
pub @get fn r<D>(self: Range<D>) -> Bounds<D> = self.hi

/// The two ends of a range as offsets, `(lo, hi)` with `lo <= hi`: what a range query walks.
/// A range written backwards, `P(3)..P(1)`, stops the program here: no query silently answers
/// as if it were empty. `l..l` is in order and empty.
pub @get fn ends<D>(self: Range<D>) -> (Int, Int):
    assert self.lo.ord <= self.hi.ord, "the range runs backwards: its end is before its start"
    (self.lo.ord, self.hi.ord)

/// First index in sorted `self` with `by(self[i]) > target`, or `none`. O(log n).
pub fn first_gt<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> ?D:
    if D.count == 0: return none
    if by(self[D.last]) <= target: return none
    var (lo, hi) = (0, D.count - 1)
    while lo < hi:
        let mid = (lo + hi) / 2
        if by(self[D.nth(mid)]) > target: hi = mid else: lo = mid + 1
    D.nth(lo)

/// Last index in sorted `self` with `by(self[i]) < target`, or `none`. O(log n).
pub fn last_lt<D, T, K>(self: [D]T, target: K, by: T -> K = x => x) -> ?D:
    if D.count == 0: return none
    if by(self[D.first]) >= target: return none
    var (lo, hi) = (0, D.count - 1)
    while lo < hi:
        let mid = (lo + hi + 1) / 2
        if by(self[D.nth(mid)]) < target: lo = mid else: hi = mid - 1
    D.nth(lo)

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

/// Number of inversions: pairs `(i, j)` with `i < j` and `a[i] > a[j]`. O(n log n).
pub fn inversions<T>(a: List<T>) -> Int:
    let n = a.len
    if n <= 1: return 0
    var cur = a.copy()
    var buf = a.copy()
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

/// Minimum of every length-`k` contiguous window of `self` (`n - k + 1` values, empty if `k > n`). O(n).
pub fn sliding_min<D, T>(self: [D]T, k: Int) -> List<T>:
    assert k >= 1, "sliding window size must be at least 1"
    var out: List<T> = []
    var dq: List<(pos: Int, val: T)> = []
    var head = 0
    var idx = 0
    for i in D:
        let x = self[i]
        while head < dq.len and dq.last!.val >= x: dq.pop()
        dq.push((pos: idx, val: x))
        if dq[head].pos <= idx - k: head += 1
        if idx >= k - 1: out.push(dq[head].val)
        idx += 1
    out

/// Maximum of every length-`k` contiguous window of `self` (`n - k + 1` values, empty if `k > n`). O(n).
pub fn sliding_max<D, T>(self: [D]T, k: Int) -> List<T>:
    assert k >= 1, "sliding window size must be at least 1"
    var out: List<T> = []
    var dq: List<(pos: Int, val: T)> = []
    var head = 0
    var idx = 0
    for i in D:
        let x = self[i]
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
/// `[if i < a.len: a[i] else: Z(0) for i in 0..size]`. O(n).
pub fn resized<T>(self: List<T>, n: Int, fill: T) -> List<T> = [
    if i < self.len: self[i] else: fill for i in 0..n
]

/// Move all elements out of `self` in O(1), leaving it empty.
pub fn take<T>(mut self: List<T>) -> List<T>:
    let out = self.copy()
    self.truncate(0)
    out

/// Insert `x` at index `i` (`0 <= i <= self.len`), shifting elements `i..` right by one. O(n).
pub fn insert<T>(mut self: List<T>, i: Int, x: T):
    assert 0 <= i and i <= self.len
    self.push(x)
    var j = self.len - 1
    while j > i:
        self.swap(j - 1, j)
        j -= 1

/// Remove and return the element at index `i` (`0 <= i < self.len`), shifting `i + 1..` left. O(n).
pub fn remove<T>(mut self: List<T>, i: Int) -> T:
    assert 0 <= i and i < self.len
    for j in i..self.len - 1:
        self.swap(j, j + 1)
    self.pop()!

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
fn binary_search_matches_a_linear_scan():
    for _ in 0..200:
        let (lo, n) = (rand(-5, 5), rand(0, 12))
        let cut = rand(lo - 1, lo + n + 1)
        let r = lo..lo + n
        assert bsearch_first(r, x => x >= cut) == first(x for x in r if x >= cut)
        assert bsearch_last(r, x => x < cut) == max(x for x in r if x < cut)

@test
fn binary_search_over_a_domain_and_a_closed_range():
    domain V = 1..=9
    let a = [V] v => v.to_idx * 3
    assert bsearch_first(V.first..=V.last, v => a[v] >= 10) == V(5)
    assert bsearch_last(V(2)..=V(6), v => a[v] <= 10) == V(4)
    assert bsearch_first(V(2)..V(4), v => a[v] >= 100) == none
    assert bsearch_first(1..=3_000_000, x => x * x >= 2_000_000) == 1415
    assert (3..9).mid == 5 and (V(2)..=V(6)).mid == V(4) and (4..4).mid == none

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
    // take moves out in O(1) leaving self empty
    var u = [1, 2, 3, 4]
    let moved = u.take()
    assert u.is_empty and moved == [1, 2, 3, 4]
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
fn bsearch_real_and_ternary_search():
    let root2 = bsearch_real(0.0, 2.0, x => x * x >= 2.0)!
    assert abs(root2 * root2 - 2.0) < 1e-12
    assert bsearch_real(0.0, 1.0, x => x * x >= 2.0) == none
    let peak_r = ternary_max_real(-5.0, 5.0, x => -(x - 1.25) * (x - 1.25))
    assert abs(peak_r - 1.25) < 1e-9
    let trough_r = ternary_min_real(-5.0, 5.0, x => (x + 2.5) * (x + 2.5))
    assert abs(trough_r + 2.5) < 1e-9
    assert ternary_max(-10..=10, x => -(x - 3) * (x - 3) + 100) == 3
    assert ternary_min(-10..=10, x => (x + 4) * (x + 4)) == -4
    assert ternary_max(5..=5, x => x) == 5
    assert ternary_max(5..5, x => x) == none
    assert ternary_min(5..5, x => x) == none
    domain P = 1..=7
    assert ternary_max(P.first..=P.last, i => -abs(i.to_idx - 4)) == P(5)
    for _ in 0..50:
        let peak = rand(-20, 20)
        assert ternary_max(-25..=25, x => -abs(x - peak)) == peak
        assert ternary_min(-25..=25, x => abs(x - peak)) == peak

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
fn ternary_search_matches_a_linear_scan():
    for _ in 0..300:
        let n = rand(1, 15)
        let peak = rand(0, n - 1)
        let plat = rand(0, n - 1 - peak)
        var a: List<Int> = []
        var v = 0
        for i in 0..n:
            if i < peak: v += rand(1, 3)
            elif i > peak + plat: v -= rand(1, 3)
            a.push(v)
        let best = ternary_max(0..n, i => a[i])!
        assert a[best] == max(x for x in a)
        let low = ternary_min(0..n, i => -a[i])!
        assert a[low] == a[best]
    assert ternary_max(0..0, i => i) == none
    domain V = 1..=9
    assert ternary_min(V.all, v => (v.to_idx - 3) * (v.to_idx - 3)) == V(4)

var level = 0
var adds: List<Int> = []
fn reset_level(): level = 0
fn add_level(t: Int): level += adds[t]
var need: List<Int> = []

@test
fn parallel_search_matches_one_search_per_query():
    for _ in 0..100:
        let n = rand(0, 12)
        adds = [rand(0, 3) for _ in 0..n]
        let q = rand(0, 10)
        need = [rand(0, 20) for _ in 0..q]
        let got = parallel_bsearch(q, 0..n, reset_level, add_level, i => level >= need[i])
        for i in 0..q:
            var s = 0
            var want: ?Int = none
            for t in 0..n:
                s += adds[t]
                if want == none and s >= need[i]: want = t
            assert got[i] == want

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

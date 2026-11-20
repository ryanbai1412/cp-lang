//! Sequences: searching, sliding, reordering and coordinate compression.
//!
//! Binary search is `bsearch_first`, `bsearch_last` and `bsearch_first_some` on a range;
//! a sorted list answers `xs.first_ge(x)` and `xs.last_le(x)`; a linear search is
//! `first(it)`, `a.find(x)`, `min_by` and `max_by`. Nothing else searches.
use std.deque.{MonoDeque}
use std.mod.{Mod}

/// The product of every value in `it`, starting from `T.one` (`1` for `Int`, `1.0` for `Float`,
/// `Mod<P>.one` for `Mod<P>`). Works on lists, arrays, ranges, and generators. O(n).
pub fn prod<T, S>(it: S) -> T:
    var acc: ?T = none
    for x in it:
        acc = if acc == none: x else: acc! * x
    acc ?? T.one

/// Every prefix folded with `f`: `[a0, f(a0, a1), f(f(a0, a1), a2), ...]`, as long as `xs`.
/// `scan(xs, +)` is the running sum and `scan(xs, max)` the running maximum; no identity
/// is needed because the first value starts the fold. O(n) calls of `f`.
pub fn scan<T>(xs: List<T>, f: (T, T) -> T) -> List<T>:
    var out: List<T> = []
    for x in xs:
        out.push(if out.len == 0: x else: f(out[out.len - 1], x))
    out

/// The first position of `r` where `ok` holds, or `none`. `ok` must be false and then true
/// along the range; the two ends are checked against that. O(log n) calls of `ok`.
pub fn bsearch_first<I>(r: Range<I>, ok: I -> Bool) -> ?I:
    if r.is_empty: return none
    let base = r.first!
    if not ok(r.last!): return none
    assert not (ok(base) and not ok(r.last!))     // `ok` must not switch back to false
    var (lo, hi) = (0, r.len - 1)                  // the answer is at an offset in lo..=hi
    while lo < hi:
        let mid = (lo + hi) / 2
        if ok((base + mid)!): hi = mid else: lo = mid + 1
    (base + lo)!

/// The last position of `r` where `ok` holds, or `none`. `ok` must be true and then false
/// along the range. O(log n) calls of `ok`.
pub fn bsearch_last<I>(r: Range<I>, ok: I -> Bool) -> ?I:
    if r.is_empty: return none
    let base = r.first!
    if not ok(base): return none
    var (lo, hi) = (0, r.len - 1)
    while lo < hi:
        let mid = (lo + hi + 1) / 2
        if ok((base + mid)!): lo = mid else: hi = mid - 1
    (base + lo)!

/// The first position of `r` where `f` gives a value, with that value: for a search whose
/// test builds the witness you want to keep. `f` must be `none` and then some along the range.
pub fn bsearch_first_some<I, W>(r: Range<I>, f: I -> ?W) -> ?(I, W):
    if r.is_empty: return none
    let base = r.first!
    var best = f(r.last!)
    if best == none: return none
    var (lo, hi) = (0, r.len - 1)
    while lo < hi:
        let mid = (lo + hi) / 2
        let got = f((base + mid)!)
        if got == none:
            lo = mid + 1
        else:
            hi = mid
            best = got
    ((base + lo)!, best!)

/// The element with the smallest `key`, the first of them on a tie; `none` for nothing.
pub fn min_by<S, T, K>(it: S, key: T -> K) -> ?T:
    var best: ?T = none
    var low: ?K = none
    for x in it:
        let k = key(x)
        if low == none or k < low!:
            best = x
            low = k
    best

/// The element with the largest `key`, the first of them on a tie; `none` for nothing.
pub fn max_by<S, T, K>(it: S, key: T -> K) -> ?T:
    var best: ?T = none
    var high: ?K = none
    for x in it:
        let k = key(x)
        if high == none or high! < k:
            best = x
            high = k
    best

/// For each left end `lo` of `r`, the nearest right end `hi >= lo` with `ok(lo, hi)`, as
/// `(lo, hi)`; left ends with no such `hi` are skipped. `ok` must be monotone: once a window
/// is good it stays good when it grows to the right, and the nearest good `hi` never moves
/// left as `lo` advances. Two pointers, O(n) calls of `ok`, no state to keep.
pub fn slide_until<I>(r: Range<I>, ok: (I, I) -> Bool) -> yields (lo: I, hi: I):
    if r.is_empty: return
    let base = r.first!
    var h = 0
    for l in 0..r.len:
        if h < l: h = l
        let lo = (base + l)!
        while h < r.len and not ok(lo, (base + h)!): h += 1
        if h == r.len: return
        yield (lo: lo, hi: (base + h)!)

fn sliding<T>(a: List<T>, k: Int, keep: (T, T) -> Bool) -> List<T>:
    assert k >= 1
    var q = MonoDeque<T>.by(keep)
    var out: List<T> = []
    for i in 0..a.len:
        q.push(i, a[i])
        q.drop_before(i - k + 1)
        if i >= k - 1: out.push(q.best!)
    out

/// The minimum of every window of `k` consecutive values: `out[i]` is `min(a[i..i + k])`. O(n).
pub fn sliding_min<T>(a: List<T>, k: Int) -> List<T> = sliding(a, k, (x, y) => x < y)

/// The maximum of every window of `k` consecutive values. O(n).
pub fn sliding_max<T>(a: List<T>, k: Int) -> List<T> = sliding(a, k, (x, y) => y < x)

/// Reverse `a` in place.
pub fn reverse<T>(mut a: List<T>):
    var (i, j) = (0, a.len - 1)
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

/// Rotate `a` left by `k` in place: what was at index `k` comes first. `k` wraps around.
pub fn rotate_left<T>(mut a: List<T>, k: Int):
    if a.is_empty: return
    let s = k % a.len
    let old = a.copy()
    for i in 0..a.len: a[i] = old[(i + s) % a.len]

/// Rotate `a` right by `k` in place: the last `k` elements move to the front. `k` wraps around.
pub fn rotate_right<T>(mut a: List<T>, k: Int) = rotate_left(mut a, -k)

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

/// The stretches of equal neighbours, in order: `runs("aabccc")` is `[('a', 2), ('b', 1), ('c', 3)]`.
pub fn runs<T>(a: List<T>) -> List<(val: T, len: Int)>:
    var out: List<(val: T, len: Int)> = []
    for x in a:
        if out.len > 0 and out[out.len - 1].val == x:
            out[out.len - 1] = (val: x, len: out[out.len - 1].len + 1)
        else:
            out.push((val: x, len: 1))
    out

/// The distinct values of a coordinate compression, in increasing order, indexed by their
/// own domain `D`. `xs[i]` is the value at index `i`, `xs.at(val)` the index of a value that
/// was compressed, and `xs.first_ge(val)` / `xs.last_le(val)` the nearest index for any value.
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
        assert i != none and vals[i!] == val      // `at` is for a value that was compressed
        i!

    /// The first index whose value is `>= val`, or `none`. O(log n).
    pub fn first_ge(self, val: T) -> ?D = vals.first_ge(val)

    /// The last index whose value is `<= val`, or `none`. O(log n).
    pub fn last_le(self, val: T) -> ?D = vals.last_le(val)

/// Coordinate compression: `let xs = compress(all)` sorts the values, drops the duplicates
/// and gives them a fresh domain `xs.D`, so an array over them is `[xs.D]Int` and an index of
/// one compression cannot be used in another. O(n log n).
pub fn compress<T>(all: List<T>):
    var sorted_vals = sorted(all)
    dedup(mut sorted_vals)
    domain D = 0..sorted_vals.len
    Compressed<D, T>([D] i => sorted_vals[i.to_idx])

// ---- v2.1 range, domain, sorted-lookup, Map 4-lookup, and container helpers

/// Left boundary cut of a range (valid even when empty).
pub @get fn l<D>(self: Range<D>) -> Bounds<D> = self.lo

/// Right boundary cut of a range (valid even when empty).
pub @get fn r<D>(self: Range<D>) -> Bounds<D> = self.hi

/// Reflect a domain position across its domain (`D.nth(D.K - 1 - p.to_idx)`).
pub @get fn mirror<D>(self: D) -> D = D.nth(D.K - 1 - self.to_idx)

/// Reflect a domain range across its domain, preserving length and emptiness.
pub fn mirror_range<D>(self: Range<D>) -> Range<D>:
    if self.is_empty: return self
    self.last!.mirror..=self.first!.mirror

/// Non-overlapping chunks of `k >= 1` positions of `D`, the last clamped to `D.last`.
pub fn chunks<D>(self: D, k: Int) -> List<Range<D>>:
    assert k >= 1
    var out: List<Range<D>> = []
    var i = 0
    while i < self.K:
        let hi = min(i + k, self.K)
        out.push(self.nth(i)..=self.nth(hi - 1))
        i = hi
    out

/// The first `min(max(k, 0), D.K)` positions of `D` as a `Range<D>` (the userland form of `D[..k]`).
pub fn prefix_range<D>(self: D, k: Int) -> Range<D>:
    let n = min(max(k, 0), self.K)
    self.bounds.nth(0)..self.bounds.nth(n)

/// Intersection of two domain ranges (the userland form of `r1 & r2`); empty when disjoint.
pub fn intersect<D>(a: Range<D>, b: Range<D>) -> Range<D>:
    let lo = if a.lo.ord >= b.lo.ord: a.lo else: b.lo
    let hi = if a.hi.ord <= b.hi.ord: a.hi else: b.hi
    if lo.ord >= hi.ord: return lo..lo
    lo..hi

/// Intersection of two integer ranges; empty when disjoint.
pub fn intersect_int(a: Range<Int>, b: Range<Int>) -> Range<Int>:
    let lo = max(a.lo, b.lo)
    let hi = min(a.hi, b.hi)
    if lo >= hi: return lo..lo
    lo..hi

/// First index in sorted list `xs` with `key(xs[i]) >= target`, or `none`. O(log n).
pub fn first_ge_by<T, K>(xs: List<T>, target: K, by: T -> K) -> ?Int:
    if xs.is_empty or by(xs[xs.len - 1]) < target: return none
    var (lo, hi) = (0, xs.len - 1)
    while lo < hi:
        let mid = (lo + hi) / 2
        if by(xs[mid]) >= target: hi = mid else: lo = mid + 1
    lo

/// Last index in sorted list `xs` with `key(xs[i]) <= target`, or `none`. O(log n).
pub fn last_le_by<T, K>(xs: List<T>, target: K, by: T -> K) -> ?Int:
    if xs.is_empty or by(xs[0]) > target: return none
    var (lo, hi) = (0, xs.len - 1)
    while lo < hi:
        let mid = (lo + hi + 1) / 2
        if by(xs[mid]) <= target: lo = mid else: hi = mid - 1
    lo

/// First `(key, val)` in `Map` with `key >= k`, or `none`.
pub fn first_ge<K, V>(self: Map<K, V>, k: K) -> ?(K, V):
    for (key, val) in self:
        if key >= k: return (key, val)
    none

/// First `(key, val)` in `Map` with `key > k`, or `none`.
pub fn first_gt<K, V>(self: Map<K, V>, k: K) -> ?(K, V):
    for (key, val) in self:
        if key > k: return (key, val)
    none

/// Last `(key, val)` in `Map` with `key <= k`, or `none`.
pub fn last_le<K, V>(self: Map<K, V>, k: K) -> ?(K, V):
    var best: ?(K, V) = none
    for (key, val) in self:
        if key <= k: best = (key, val)
        else: break
    best

/// Last `(key, val)` in `Map` with `key < k`, or `none`.
pub fn last_lt<K, V>(self: Map<K, V>, k: K) -> ?(K, V):
    var best: ?(K, V) = none
    for (key, val) in self:
        if key < k: best = (key, val)
        else: break
    best

/// Shorthand aliases for the 4 ordered `Map` lookups (`ge`, `gt`, `le`, `lt`).
pub fn ge<K, V>(self: Map<K, V>, k: K) -> ?(K, V) = first_ge(self, k)
pub fn gt<K, V>(self: Map<K, V>, k: K) -> ?(K, V) = first_gt(self, k)
pub fn le<K, V>(self: Map<K, V>, k: K) -> ?(K, V) = last_le(self, k)
pub fn lt<K, V>(self: Map<K, V>, k: K) -> ?(K, V) = last_lt(self, k)

/// Swap two positions of a domain array in place; safely a no-op when `i == j`.
pub fn swap_at<D, T>(mut self: [D]T, i: D, j: D):
    if i == j: return
    let t = self[i]
    self[i] = self[j]
    self[j] = t

/// Move all elements out of `self`, leaving it empty.
pub fn take<T>(mut self: List<T>) -> List<T>:
    let out = self.copy()
    self.truncate(0)
    out

/// Drain all elements of `other` into `self`, leaving `other` empty.
pub fn absorb<T>(mut self: List<T>, mut other: List<T>):
    for x in other: self.push(x)
    other.truncate(0)

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

@test
fn binary_search_matches_a_linear_scan():
    for _ in 0..200:
        let (lo, n) = (rand(-5, 5), rand(0, 12))
        let cut = rand(lo - 1, lo + n + 1)
        let r = lo..lo + n
        assert bsearch_first(r, x => x >= cut) == first(x for x in r if x >= cut)
        assert bsearch_last(r, x => x < cut) == max(x for x in r if x < cut)
        let hit = bsearch_first_some(r, x => if x >= cut: x * x else: none)
        if n == 0 or cut >= lo + n: assert hit == none
        else: assert hit == (max(cut, lo), max(cut, lo) * max(cut, lo))

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
    let xs = [1, 3, 3, 7, 10]
    assert xs.first_ge(3) == 1 and xs.first_ge(4) == 3 and xs.first_ge(11) == none
    assert xs.last_le(3) == 2 and xs.last_le(0) == none and xs.last_le(99) == 4
    assert xs.find(7) == 3 and xs.find(8) == none
    domain V = 1..=5
    let a = [V] v => xs[v.to_idx]
    assert a.first_ge(4) == V(4) and a.last_le(3) == V(3) and a.find(10) == V(5)
    assert first(x for x in xs if x > 3) == 7 and first(x for x in xs if x > 10) == none
    let words = ["pear", "fig", "banana", "kiwi"]
    assert min_by(words, w => w.len) == "fig" and max_by(words, w => w.len) == "banana"
    let nothing: List<Str> = []
    assert min_by(nothing, w => w.len) == none

@test
fn slide_until_finds_the_nearest_good_right_end():
    for _ in 0..100:
        let n = rand(0, 12)
        let a = [rand(0, 9) for _ in 0..n]
        let need = rand(1, 25)
        // the shortest window starting at each `lo` whose sum reaches `need`
        var pre = [0]
        for x in a: pre.push(pre[pre.len - 1] + x)
        let got = [w for w in slide_until(0..n, (l, r) => pre[r + 1] - pre[l] >= need)]
        var want: List<(Int, Int)> = []
        for l in 0..n:
            let hi = first(r for r in l..n if pre[r + 1] - pre[l] >= need)
            if hi != none: want.push((l, hi!))
        assert [(w.lo, w.hi) for w in got] == want

@test
fn sliding_minimum_and_maximum():
    assert sliding_min([4, 2, 7, 1, 5], 2) == [2, 2, 1, 1]
    assert sliding_max([4, 2, 7, 1, 5], 3) == [7, 7, 7]
    for _ in 0..80:
        let a = [rand(-9, 9) for _ in 0..rand(1, 14)]
        let k = rand(1, a.len)
        assert sliding_min(a, k) == [min(a[j] for j in i..i + k)! for i in 0..=a.len - k]
        assert sliding_max(a, k) == [max(a[j] for j in i..i + k)! for i in 0..=a.len - k]
    domain D = 0..5
    assert [w.len for w in D.windows(3)] == [3, 3, 3] and D.windows(3)[1] == (D(1)..=D(3))

@test
fn reorder_in_place():
    var a = [1, 1, 2, 3, 3, 3, 1]
    dedup(mut a)
    assert a == [1, 2, 3, 1]
    rotate_left(mut a, 1)
    assert a == [2, 3, 1, 1]
    rotate_left(mut a, -1)
    assert a == [1, 2, 3, 1]
    reverse(mut a)
    assert a == [1, 3, 2, 1]
    var p = [1, 2, 2]
    var seen = [p.copy()]
    while next_permutation(mut p): seen.push(p.copy())
    assert seen == [[1, 2, 2], [2, 1, 2], [2, 2, 1]] and p == [1, 2, 2]
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
fn prod_and_min_max_by_tie_breaking():
    assert prod([2, 3, 4]) == 24
    assert prod(x for x in 1..=5) == 120
    assert prod::<Int>([]) == 1
    type Z = Mod<1000000007>
    assert prod([Z(2), Z(3), Z(5)]) == Z(30)
    assert prod::<Z>([]) == Z.one
    // First extremum always wins ties; empty input always returns none
    let items = [(10, "first_min"), (5, "a"), (5, "b"), (20, "first_max"), (20, "second_max"), (5, "c")]
    assert min_by(items, p => p.0) == (5, "a")
    assert max_by(items, p => p.0) == (20, "first_max")
    assert min_by([("x", 1), ("y", 1), ("z", 1)], p => p.1) == ("x", 1)
    assert max_by([("x", 1), ("y", 1), ("z", 1)], p => p.1) == ("x", 1)
    let empty: List<(Int, Str)> = []
    assert min_by(empty, p => p.0) == none and max_by(empty, p => p.0) == none
    assert min_by([x for x in items if x.0 < 0], p => p.0) == none
    assert max_by([x for x in items if x.0 < 0], p => p.0) == none

@test
fn chunks_prefix_intersect_bounds_mirror_and_rotate_right():
    domain D = 1..=10
    let cs = D.chunks(3)
    assert [c.len for c in cs] == [3, 3, 3, 1]
    assert cs[0] == (D(1)..=D(3)) and cs[3] == (D(10)..=D(10))
    assert D.chunks(20) == [D(1)..=D(10)]
    // Range.l and Range.r boundary cuts even when empty
    let empty = D(4)..D(4)
    assert empty.l == D(4).l and empty.r == D(4).l and empty.l == empty.r
    let seg = D(2)..=D(5)
    assert seg.l == D(2).l and seg.r == D(5).r
    // D prefix range (D[..k])
    assert D.prefix_range(0).is_empty
    assert D.prefix_range(4) == (D(1)..=D(4))
    assert D.prefix_range(99) == (D(1)..=D(10))
    // Range intersection (r1 & r2)
    assert intersect(D(2)..=D(7), D(5)..=D(9)) == (D(5)..=D(7))
    assert intersect(D(1)..=D(3), D(6)..=D(8)).is_empty
    assert intersect_int(2..8, 5..12) == (5..8)
    assert intersect_int(1..3, 3..6).is_empty
    // Mirror on positions and ranges
    assert D(1).mirror == D(10) and D(3).mirror == D(8) and D(3).mirror.mirror == D(3)
    assert mirror_range(D(1)..=D(3)) == (D(8)..=D(10))
    assert mirror_range(D(4)..D(4)).is_empty
    // rotate_right
    var xs = [1, 2, 3, 4, 5]
    rotate_right(mut xs, 2)
    assert xs == [4, 5, 1, 2, 3]
    rotate_right(mut xs, 7)
    assert xs == [2, 3, 4, 5, 1]
    // first_ge_by / last_le_by
    let pairs = [(1, "a"), (3, "b"), (3, "c"), (7, "d")]
    assert first_ge_by(pairs, 3, by: p => p.0) == 1
    assert first_ge_by(pairs, 4, by: p => p.0) == 3
    assert first_ge_by(pairs, 9, by: p => p.0) == none
    assert last_le_by(pairs, 3, by: p => p.0) == 2
    assert last_le_by(pairs, 0, by: p => p.0) == none

@test
fn map_four_lookups_and_container_helpers():
    var m = Map<Int, Str>()
    m[10] = "ten"
    m[20] = "twenty"
    m[30] = "thirty"
    assert m.first_ge(20) == (20, "twenty") and m.ge(21) == (30, "thirty") and m.ge(31) == none
    assert m.first_gt(20) == (30, "thirty") and m.gt(10) == (20, "twenty") and m.gt(30) == none
    assert m.last_le(20) == (20, "twenty") and m.le(19) == (10, "ten") and m.le(5) == none
    assert m.last_lt(20) == (10, "ten") and m.lt(30) == (20, "twenty") and m.lt(10) == none
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
            let want_le = max_by([(k, rm[k]) for k in ks if k <= q], p => p.0)
            let want_lt = max_by([(k, rm[k]) for k in ks if k < q], p => p.0)
            assert rm.first_ge(q) == want_ge and rm.ge(q) == want_ge
            assert rm.first_gt(q) == want_gt and rm.gt(q) == want_gt
            assert rm.last_le(q) == want_le and rm.le(q) == want_le
            assert rm.last_lt(q) == want_lt and rm.lt(q) == want_lt
    // swap_at (self-swap safe), take, absorb
    domain I = 0..4
    var a = [I] i => i.to_idx * 10
    a.swap_at(I(1), I(3))
    a.swap_at(I(2), I(2))
    assert [a[i] for i in I] == [0, 30, 20, 10]
    var u = [1, 2]
    var v = [3, 4, 5]
    u.absorb(mut v)
    assert v.is_empty and u == [1, 2, 3, 4, 5]
    let moved = u.take()
    assert u.is_empty and moved == [1, 2, 3, 4, 5]

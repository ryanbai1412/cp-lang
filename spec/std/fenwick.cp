//! Fenwick tree (binary indexed tree) over a domain: point add, range sum.
use std.algebra.{Group}
use std.seq.{deduped, ends}

/// Point add and range sum over anything with `+` and `-` (`Int`, `Mod<P>`).
/// `Fenwick<D, Int>()` starts at zero and `Fenwick<D, Int>(a)` from an array, in O(n).
/// `add`, `prefix` and `sum` are O(log n); memory is n + 1 values.
/// A range is one prefix minus another, so there is no "Fenwick on max": that is a `SegTree`.
pub struct Fenwick<D, T>:
    t: List<T>

    /// A Fenwick tree over `D`: all zeros, or holding the values of `a` when given. The values
    /// must form a group under `+` and `-` (`Int`, `Mod<P>`, `F64`), since a range sum is one
    /// prefix minus another.
    ///
    /// cost: O(|D|) time; memory |D| + 1 values
    /// empty: an empty `D` gives a tree whose every sum is zero
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: never
    /// example: `Fenwick([1, 2, 3]).all == 6`
    /// example: `Fenwick<0..3, Int>().all == 0`
    pub fn new(a: ?[D]T = None) -> Fenwick<D, T>:
        var t = List.filled(D.K + 1, T.zero)
        if let init = a:
            for i in D:
                let p = i.to_idx + 1
                t[p] = t[p] + init[i]
                let q = p + (p & -p)
                if q <= D.K: t[q] = t[q] + t[p]
        Fenwick(t)

    /// Add `x` to the value at position `i`: `a[i] += x`.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `var f = Fenwick([1, 2, 3]); f.add(0, 5); f.fold(0..2) == 8`
    pub fn add(mut self, i: D, x: T):
        var p = i.to_idx + 1
        while p < t.len:
            t[p] = t[p] + x
            p += p & -p

    /// The sum of every position left of the boundary `b`. For a position `v`, `prefix(v.r)`
    /// includes `v` and `prefix(v.l)` does not; the start cut gives zero.
    ///
    /// cost: O(log |D|)
    /// traps: never (`b` is a cut of `D` by type)
    /// example: `Fenwick([1, 2, 3]).prefix((0..2).r) == 3`
    /// example: `Fenwick([1, 2, 3]).prefix((0..0).r) == 0`
    pub fn prefix(self, b: Cut<D>) -> T:
        var p = b.ord
        var acc = T.zero
        while p > 0:
            acc = acc + t[p]
            p -= p & -p
        acc

    /// The sum over the positions of `r`: `prefix(r.r) - prefix(r.l)`. Accepts every range form
    /// (`l..r`, `l..=r`, `..i`, `i..`).
    ///
    /// cost: O(log |D|)
    /// empty: zero
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `Fenwick([1, 2, 3]).fold(1..3) == 5`
    /// example: `Fenwick([1, 2, 3]).fold(2..2) == 0`
    pub fn fold(self, r: Range<D>) -> T:
        self.prefix(r.end) - self.prefix(r.start)

    /// The smallest boundary `b` with `prefix(b) >= target`, by binary lifting over the tree;
    /// `None` when even the whole sum is below `target`. A `target <= 0` gives the start cut.
    /// Correct only when every value is non-negative (prefix sums non-decreasing).
    ///
    /// cost: O(log |D|)
    /// ties: the leftmost qualifying cut
    /// traps: never (negative values give an unspecified cut, not a trap)
    /// example: `Fenwick([1, 2, 3]).first_ge(3) == Some((0..2).r)`
    /// example: `Fenwick([1, 2, 3]).first_ge(7) == None`
    /// example: `Fenwick([1, 2, 3]).first_ge(0) == Some((0..0).r)`
    pub fn first_ge(self, target: Int) -> ?Cut<D>:
        if target <= 0: return D.cuts.nth(0)
        var p = 0
        var rest = target
        var step = bit_ceil(D.K + 1) / 2
        while step > 0:
            if p + step <= D.K:
                // The walk is a binary search, so it is wrong once a block sums below zero.
                assert t[p + step] >= 0, "`first_ge` and `kth` need non-negative values"
                if t[p + step] < rest:
                    p += step
                    rest -= t[p]
            step /= 2
        if p == D.K: None else: D.cuts.nth(p + 1)

    /// The `k`-th item counting from 1, when position `d` holds `a[d]` items: the first `d` with
    /// `prefix(d.r) >= k`, or `None` when fewer than `k` items exist in total. Positions with value
    /// zero are never returned. For non-negative values.
    ///
    /// cost: O(log |D|)
    /// ties: the leftmost qualifying position
    /// traps: when `k < 1` (negative values give an unspecified position, not a trap)
    /// example: `Fenwick([1, 0, 2]).kth(2) == Some(2)`
    /// example: `Fenwick([1, 0, 2]).kth(1) == Some(0)`
    /// example: `Fenwick([1, 0, 2]).kth(4) == None`
    pub fn kth(self, k: Int) -> ?D:
        assert k >= 1, "`kth` counts from 1"
        let b = self.first_ge(k)
        if b == None: None else: b!.left

    /// The current value at position `i`: `sum(i..=i)`.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `Fenwick([1, 2, 3]).get(1) == 2`
    pub fn get(self, i: D) -> T:
        self.prefix(i.r) - self.prefix(i.l)

    /// Assign `a[i] = x`, by adding the difference `x - get(i)`; also the write-through place for
    /// `f[i] = x`.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `var f = Fenwick([1, 2, 3]); f.set(1, 5); f.all == 9`
    pub fn set(mut self, i: D, x: T):
        self.add(i, x - self.get(i))

    /// The sum of every position of `D`, kept current by `add`, `set` and `clear`.
    ///
    /// cost: O(1)
    /// empty: zero on an empty domain
    /// traps: never
    /// example: `Fenwick([1, 2, 3]).all == 6`
    @get pub fn all(self) -> T:
        self.prefix(D.cuts.nth(D.K))

    /// The sum of every position right of the boundary `b`: `all - prefix(b)`. The end cut gives zero.
    ///
    /// cost: O(log |D|)
    /// traps: never
    /// example: `Fenwick([1, 2, 3]).suffix((0..1).r) == 5`
    /// example: `Fenwick([1, 2, 3]).suffix((0..3).r) == 0`
    pub fn suffix(self, b: Cut<D>) -> T:
        self.all - self.prefix(b)

    /// The smallest boundary `b` with `prefix(b) > target`, or `None` when the whole sum is at most
    /// `target`. For non-negative values.
    ///
    /// cost: O(log |D|)
    /// ties: the leftmost qualifying cut
    /// traps: never (negative values give an unspecified cut, not a trap)
    /// example: `Fenwick([1, 2, 3]).first_gt(3) == Some((0..3).r)`
    /// example: `Fenwick([1, 2, 3]).first_gt(6) == None`
    pub fn first_gt(self, target: Int) -> ?Cut<D>:
        if target < 0: return D.cuts.nth(0)
        var p = 0
        var rest = target
        var step = bit_ceil(D.K + 1) / 2
        while step > 0:
            if p + step <= D.K:
                assert t[p + step] >= 0, "`first_gt` needs non-negative values"
                if t[p + step] <= rest:
                    p += step
                    rest -= t[p]
            step /= 2
        if p == D.K: None else: D.cuts.nth(p + 1)

    /// Reset every value to zero, keeping the domain and the allocation.
    ///
    /// cost: O(|D|)
    /// traps: never
    /// example: `var f = Fenwick([1, 2, 3]); f.clear(); f.all == 0`
    pub fn clear(mut self):
        for p in 0..t.len: t[p] = T.zero

/// Point add and rectangle sum on a grid. `add` and `sum` are O(log R * log C);
/// memory is (R + 1) * (C + 1) values.
pub struct Fenwick2D<R, C, T>:
    t: List<T>

    /// An all-zero 2-D Fenwick tree over the grid `R` by `C`.
    ///
    /// cost: O(|R| |C|) time; memory (|R| + 1)(|C| + 1) values
    /// traps: never
    /// example: `Fenwick2D<0..2, 0..2, Int>().fold(0..2, 0..2) == 0`
    pub fn new() -> Fenwick2D<R, C, T> = Fenwick2D(List.filled((R.K + 1) * (C.K + 1), T.zero))

    /// Add `x` to the cell `(r, c)`: `a[r, c] += x`.
    ///
    /// cost: O(log |R| log |C|)
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(1, 0, 5); f.fold(1..2, 0..1) == 5`
    pub fn add(mut self, r: R, c: C, x: T):
        var p = r.to_idx + 1
        while p <= R.K:
            var q = c.to_idx + 1
            while q <= C.K:
                t[p * (C.K + 1) + q] = t[p * (C.K + 1) + q] + x
                q += q & -q
            p += p & -p

    fn corner(self, r: Int, c: Int) -> T:
        var acc = T.zero
        var p = r
        while p > 0:
            var q = c
            while q > 0:
                acc = acc + t[p * (C.K + 1) + q]
                q -= q & -q
            p -= p & -p
        acc

    /// The sum over the rectangle `rows` by `cols`, by inclusion-exclusion over four prefixes.
    ///
    /// cost: O(log |R| log |C|)
    /// empty: zero when either range is empty
    /// traps: when a range of a non-empty rectangle reaches outside its domain
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(0, 0, 1); f.add(1, 1, 2); f.fold(0..2, 1..2) == 2`
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(0, 0, 1); f.fold(0..2, 1..1) == 0`
    pub fn fold(self, rows: Range<R>, cols: Range<C>) -> T:
        let r0 = rows.start.ord
        let r1 = rows.end.ord
        let c0 = cols.start.ord
        let c1 = cols.end.ord
        self.corner(r1, c1) - self.corner(r0, c1) - self.corner(r1, c0) + self.corner(r0, c0)

    /// The sum of every cell above the row cut `r` and left of the column cut `c`.
    ///
    /// cost: O(log |R| log |C|)
    /// traps: never
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(0, 0, 1); f.add(1, 1, 2); f.prefix((0..1).r, (0..2).r) == 1`
    pub fn prefix(self, r: Cut<R>, c: Cut<C>) -> T:
        self.corner(r.ord, c.ord)

    /// The current value of the cell `(r, c)`.
    ///
    /// cost: O(log |R| log |C|)
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(1, 1, 2); f.get(1, 1) == 2`
    pub fn get(self, r: R, c: C) -> T:
        let p = r.to_idx
        let q = c.to_idx
        self.corner(p + 1, q + 1) - self.corner(p, q + 1) - self.corner(p + 1, q) + self.corner(p, q)

    /// Reset every cell to zero, keeping the domains and the allocation.
    ///
    /// cost: O(|R| |C|)
    /// traps: never
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(1, 1, 2); f.clear(); f.get(1, 1) == 0`
    pub fn clear(mut self):
        for p in 0..t.len: t[p] = T.zero

/// Range add and point get via a difference Fenwick tree.
/// `DualFenwick<D, Int>()` starts at zero and `DualFenwick<D, Int>(a)` from an array, in O(n).
/// `add` and `get` are O(log n); memory is n + 1 values.
pub struct DualFenwick<D, T>:
    t: List<T>

    /// A range-add, point-get Fenwick tree over `D`: all zeros, or holding the values of `a` when
    /// given. It stores differences, so `get(i)` is a prefix sum of them.
    ///
    /// cost: O(|D|) time; memory |D| + 1 values
    /// empty: an empty `D` gives a tree with no positions to read
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: never
    /// example: `DualFenwick([1, 2, 3]).get(2) == 3`
    pub fn new(a: ?[D]T = None) -> DualFenwick<D, T>:
        var f = DualFenwick(List.filled(D.K + 1, T.zero))
        if let init = a:
            var prev = T.zero
            for i in D:
                let diff = init[i] - prev
                prev = init[i]
                let p = i.to_idx + 1
                f.t[p] = f.t[p] + diff
                let q = p + (p & -p)
                if q <= D.K: f.t[q] = f.t[q] + f.t[p]
        f

    fn add_at(mut self, p0: Int, x: T):
        var p = p0
        while p < t.len:
            t[p] = t[p] + x
            p += p & -p

    /// Add `x` to every position of `r`.
    ///
    /// cost: O(log |D|)
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var d = DualFenwick([1, 2, 3]); d.add(0..2, 10); d.get(1) == 12`
    /// example: `var d = DualFenwick([1, 2, 3]); d.add(0..2, 10); d.get(2) == 3`
    pub fn add(mut self, r: Range<D>, x: T):
        let (lo, hi) = r.ends
        if lo == hi: return
        self.add_at(lo + 1, x)
        if hi < D.K: self.add_at(hi + 1, T.zero - x)

    /// The current value at position `i`: its initial value plus every `x` added over a range
    /// containing `i`.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `var d = DualFenwick([1, 2, 3]); d.add(1..3, 5); d.get(1) == 7`
    pub fn get(self, i: D) -> T:
        var p = i.to_idx + 1
        var acc = T.zero
        while p > 0:
            acc = acc + t[p]
            p -= p & -p
        acc

    /// Reset every value to zero, keeping the domain and the allocation.
    ///
    /// cost: O(|D|)
    /// traps: never
    /// example: `var d = DualFenwick([1, 2, 3]); d.clear(); d.get(0) == 0`
    pub fn clear(mut self):
        for p in 0..t.len: t[p] = T.zero

/// Range add and range sum over `Int` with two difference Fenwick trees (`d[i]` and
/// `i * d[i]`): `RangeFenwick<D>()` starts at zero and `RangeFenwick<D>(a)` from an array.
/// `add`, `prefix` and `fold` are O(log n); memory is 3(n + 1) values. Smaller constants than a
/// `LazySegTree` over `AddSum`.
pub struct RangeFenwick<D>:
    base: List<Int>
    b1: List<Int>
    b2: List<Int>

    /// A range-add, range-sum tree over `D`: all zeros, or holding the values of `a` when given.
    ///
    /// cost: O(|D|) time; memory 3(|D| + 1) values
    /// empty: an empty `D` gives a tree whose every sum is zero
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: never
    /// example: `RangeFenwick([1, 2, 3]).fold(0..3) == 6`
    /// example: `RangeFenwick<0..3>().fold(0..3) == 0`
    pub fn new(a: ?[D]Int = None) -> RangeFenwick<D>:
        var base = List.filled(D.K + 1, 0)
        if let init = a:
            for i in D: base[i.to_idx + 1] = base[i.to_idx] + init[i]
        RangeFenwick(base, List.filled(D.K + 1, 0), List.filled(D.K + 1, 0))

    fn bump(mut self, p0: Int, x: Int):
        var p = p0 + 1
        while p <= D.K:
            b1[p] += x
            b2[p] += x * p0
            p += p & -p

    /// Add `x` to every position of `r`.
    ///
    /// cost: O(log |D|)
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var f = RangeFenwick([1, 2, 3]); f.add(0..2, 10); f.fold(0..3) == 26`
    /// example: `var f = RangeFenwick([1, 2, 3]); f.add(1..3, 5); f.fold(2..3) == 8`
    pub fn add(mut self, r: Range<D>, x: Int):
        let lo = r.start.ord
        let hi = r.end.ord
        if lo >= hi: return
        self.bump(lo, x)
        if hi < D.K: self.bump(hi, -x)

    /// The sum of every position left of the boundary `b`, adds included.
    ///
    /// cost: O(log |D|)
    /// traps: never (`b` is a cut of `D` by type)
    /// example: `var f = RangeFenwick([1, 2, 3]); f.add(0..3, 1); f.prefix((0..2).r) == 5`
    pub fn prefix(self, b: Cut<D>) -> Int:
        let k = b.ord
        var s1 = 0
        var s2 = 0
        var p = k
        while p > 0:
            s1 += b1[p]
            s2 += b2[p]
            p -= p & -p
        base[k] + s1 * k - s2

    /// The sum over the positions of `r`.
    ///
    /// cost: O(log |D|)
    /// empty: zero
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var f = RangeFenwick([1, 2, 3]); f.add(1..2, 4); f.fold(1..3) == 9`
    /// example: `RangeFenwick([1, 2, 3]).fold(2..2) == 0`
    pub fn fold(self, r: Range<D>) -> Int:
        self.prefix(r.end) - self.prefix(r.start)

    /// Reset every value to zero, keeping the domain and the allocation.
    ///
    /// cost: O(|D|)
    /// traps: never
    /// example: `var f = RangeFenwick([1, 2, 3]); f.clear(); f.fold(0..3) == 0`
    pub fn clear(mut self):
        for p in 0..base.len:
            base[p] = 0
            b1[p] = 0
            b2[p] = 0

/// Point xor and range xor: `XorFenwick<D, T>()` starts at zero and `XorFenwick<D, T>(a)` from an
/// array, in O(n). `flip`, `prefix` and `xor` are O(log n); memory is n + 1 values. `^` is its
/// own inverse, so a range is one prefix xored with another.
pub struct XorFenwick<D, T>:
    t: List<T>

    /// A xor Fenwick tree over `D`: all zeros, or holding the values of `a` when given.
    ///
    /// cost: O(|D|) time; memory |D| + 1 values
    /// empty: an empty `D` gives a tree whose every xor is zero
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: never
    /// example: `XorFenwick([1, 2, 4]).xor(0..3) == 7`
    pub fn new(a: ?[D]T = None) -> XorFenwick<D, T>:
        var t = List.filled(D.K + 1, T.zero)
        if let init = a:
            for i in D:
                let p = i.to_idx + 1
                t[p] ^= init[i]
                let q = p + (p & -p)
                if q <= D.K: t[q] = t[q] ^ t[p]
        XorFenwick(t)

    /// Xor `x` into the value at position `i`: `a[i] ^= x`.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `var f = XorFenwick([1, 2, 4]); f.flip(1, 2); f.xor(0..3) == 5`
    pub fn flip(mut self, i: D, x: T):
        var p = i.to_idx + 1
        while p < t.len:
            t[p] ^= x
            p += p & -p

    /// The xor of every position left of the boundary `b`.
    ///
    /// cost: O(log |D|)
    /// traps: never (`b` is a cut of `D` by type)
    /// example: `XorFenwick([1, 2, 4]).prefix((0..2).r) == 3`
    pub fn prefix(self, b: Cut<D>) -> T:
        var p = b.ord
        var acc = T.zero
        while p > 0:
            acc = acc ^ t[p]
            p -= p & -p
        acc

    /// The xor over the positions of `r`.
    ///
    /// cost: O(log |D|)
    /// empty: zero
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `XorFenwick([1, 2, 4]).xor(1..3) == 6`
    /// example: `XorFenwick([1, 2, 4]).xor(2..2) == 0`
    pub fn xor(self, r: Range<D>) -> T:
        self.prefix(r.end) ^ self.prefix(r.start)

/// O(n log n) space. Offline coordinate-compressed 2D Fenwick tree (KACTL `FT2`):
/// register all update coordinates with `add_point`, call `build()`, then run `add` and `sum`
/// in O(log^2 n).
fn lower_bound(a: List<Int>, x: Int) -> Int:
    var lo = 0
    var hi = a.len
    while lo < hi:
        let mid = (lo + hi) / 2
        if a[mid] < x: lo = mid + 1
        else: hi = mid
    lo

pub struct SparseFenwick2D<T>:
    n: Int
    pts: List<(Int, Int)>
    xs: List<Int>
    ys: List<List<Int>>
    ft: List<List<T>>
    built: Bool

    /// An empty offline 2-D Fenwick builder. Use it in two phases: register every point that will
    /// ever be updated with `add_point`, call `build` once, then use `add` and `sum`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = SparseFenwick2D<Int>(); f.build(); f.fold(0..10, 0..10) == 0`
    pub fn new() -> SparseFenwick2D<T>:
        SparseFenwick2D(0, [], [], [], [], false)

    /// Register the point `(x, y)` as a future `add` target. Registering a point twice is harmless.
    ///
    /// cost: O(1) amortized
    /// traps: after `build`
    /// example: `var f = SparseFenwick2D<Int>(); f.add_point(5, 7); f.add_point(5, 7); f.build(); f.add(5, 7, 3); f.fold(0..10, 0..10) == 3`
    pub fn add_point(mut self, x: Int, y: Int):
        assert not built, "`add_point` after `build`"
        pts.push((x, y))

    /// Compress the registered `x` coordinates, and per `x` bucket the `y` coordinates it can
    /// receive, and allocate the bucket trees at zero. Ends the registration phase.
    ///
    /// cost: O(n log n) for `n` registered points; memory O(n log n)
    /// traps: when called a second time
    /// example: `var f = SparseFenwick2D<Int>(); f.add_point(1, 2); f.build(); f.fold(1..2, 2..3) == 0`
    pub fn build(mut self):
        assert not built, "`build` called twice"
        built = true
        var all_x: List<Int> = []
        for (x, _) in pts: all_x.push(x)
        xs = all_x.sorted().deduped()
        n = xs.len
        ys = List.filled(n + 1, [])
        for (x, y) in pts:
            var i = lower_bound(xs, x) + 1
            while i <= n:
                ys[i].push(y)
                i += i & -i
        ft = List.filled(n + 1, [])
        for i in 0..=n:
            ys[i] = ys[i].sorted().deduped()
            ft[i] = List.filled(ys[i].len + 1, T.zero)

    /// Add `v` at the registered point `(x, y)`.
    ///
    /// cost: O(log^2 n)
    /// traps: before `build`, or when `(x, y)` was never registered
    /// example: `var f = SparseFenwick2D<Int>(); f.add_point(1, 2); f.add_point(3, 4); f.build(); f.add(1, 2, 5); f.add(3, 4, 7); f.fold(0..2, 0..10) == 5`
    pub fn add(mut self, x: Int, y: Int, v: T):
        assert built, "`add` before `build`"
        var i = lower_bound(xs, x)
        assert i < n and xs[i] == x, "`add` at a point that was never registered"
        i += 1
        while i <= n:
            var j = lower_bound(ys[i], y)
            assert j < ys[i].len and ys[i][j] == y, "`add` at a point that was never registered"
            j += 1
            while j <= ys[i].len:
                ft[i][j] = ft[i][j] + v
                j += j & -j
            i += i & -i

    fn corner(self, x: Int, y: Int) -> T:
        var acc = T.zero
        var i = lower_bound(xs, x)
        while i > 0:
            var j = lower_bound(ys[i], y)
            while j > 0:
                acc = acc + ft[i][j]
                j -= j & -j
            i -= i & -i
        acc

    /// The total added at registered points with `x` in `xs` and `y` in `ys`. The ranges are over
    /// raw coordinates; points outside them, or never registered, contribute nothing.
    ///
    /// cost: O(log^2 n)
    /// empty: zero when either range is empty
    /// traps: before `build`
    /// example: `var f = SparseFenwick2D<Int>(); f.add_point(1, 2); f.add_point(3, 4); f.build(); f.add(1, 2, 5); f.add(3, 4, 7); f.fold(0..10, 3..5) == 7`
    /// example: `var f = SparseFenwick2D<Int>(); f.add_point(1, 2); f.build(); f.add(1, 2, 5); f.fold(2..2, 0..10) == 0`
    pub fn fold(self, xs: Range<Int>, ys: Range<Int>) -> T:
        assert built, "`fold` before `build`"
        if xs.start >= xs.end or ys.start >= ys.end: return T.zero
        self.corner(xs.end, ys.end) - self.corner(xs.start, ys.end) - self.corner(xs.end, ys.start) + self.corner(xs.start, ys.start)

@test
fn sums_match_a_plain_array():
    domain V = 0..37
    var f = Fenwick<V, Int>()
    var a = List.filled(37, 0)
    var seed = 7
    for _ in 0..300:
        seed = (seed * 1103515245 + 12345) % 2147483648
        let (i, x) = (seed % 37, seed / 37 % 21 - 10)
        f.add(V(i), x)
        a[i] += x
        let lo = seed / 1000 % 37
        let hi = lo + seed / 100000 % (38 - lo)
        var want = 0
        for j in lo..hi: want += a[j]
        assert f.fold(V.cuts.nth(lo)..V.cuts.nth(hi)) == want
        assert f.prefix(V.cuts.nth(hi)) == f.fold(V.cuts.nth(0)..V.cuts.nth(hi))
        assert f.get(V(i)) == a[i]

@test
fn empty_and_half_open_ranges():
    domain V = 0..5
    let a: [V]Int = [3, 1, 4, 1, 5]
    var f = Fenwick<V, Int>(a)
    assert f.fold(V(2)..V(2)) == 0
    assert f.fold(V(1)..V(3)) == 5
    assert f.fold(V(1)..=V(3)) == 6
    assert f.prefix(V.cuts.nth(0)) == 0
    assert f.suffix(V.cuts.nth(5)) == 0
    assert f.suffix(V.cuts.nth(3)) == 6
    f.set(V(4), 0)
    assert f.fold(V(0)..=V(4)) == 9
    domain E = 0..0
    let g = Fenwick<E, Int>()
    assert g.prefix(E.cuts.nth(0)) == 0

@test
fn first_ge_finds_the_kth():
    domain V = 0..8
    var f = Fenwick<V, Int>()
    for v in [1, 4, 4, 6]: f.add(V(v), 1)
    assert f.first_ge(1)!.left == V(1)
    assert f.first_ge(3)!.left == V(4)
    assert f.first_ge(4)!.left == V(6)
    assert f.first_ge(5) == None
    assert f.kth(1) == V(1)
    assert f.kth(3) == V(4)
    assert f.kth(4) == V(6)
    assert f.kth(5) == None

@test
fn two_dimensions_match_a_plain_grid():
    domain R = 0..6
    domain C = 0..9
    var f = Fenwick2D<R, C, Int>()
    var g = List.filled(54, 0)
    var seed = 11
    for _ in 0..200:
        seed = (seed * 1103515245 + 12345) % 2147483648
        let (r, c, x) = (seed % 6, seed / 6 % 9, seed / 54 % 9 - 4)
        f.add(R(r), C(c), x)
        g[r * 9 + c] += x
        let (r0, c0) = (seed / 1000 % 6, seed / 7000 % 9)
        let (r1, c1) = (r0 + seed / 70000 % (7 - r0), c0 + seed / 700000 % (10 - c0))
        var want = 0
        for i in r0..r1:
            for j in c0..c1: want += g[i * 9 + j]
        assert f.fold(R.cuts.nth(r0)..R.cuts.nth(r1), C.cuts.nth(c0)..C.cuts.nth(c1)) == want
        assert f.get(R(r), C(c)) == g[r * 9 + c]

@test
fn xor_matches_a_plain_array():
    domain I = 0..12
    var f = Fenwick<I, XorTag>()
    var a: [I]Int = 0
    for _ in 0..300:
        let i = I(rand(0, 11))
        let x = rand(0, 1023)
        f.add(i, XorTag(x))
        a[i] ^= x
        let l = rand(0, 12)
        let r = rand(l, 12)
        var want = 0
        for k in l..r: want ^= a[I(k)]
        assert f.fold(I.cuts.nth(l)..I.cuts.nth(r)).v == want

struct XorTag(v: Int):
    const zero = XorTag(0)

impl Add for XorTag:
    fn add(a, b) = XorTag(a.v ^ b.v)

impl Sub for XorTag:
    fn sub(a, b) = XorTag(a.v ^ b.v)

@test
fn dual_and_range_fenwick_match_brute():
    domain V = 0..23
    var f = DualFenwick<V, Int>()
    var a = List.filled(23, 0)
    var seed = 5
    for _ in 0..300:
        seed = (seed * 1103515245 + 12345) % 2147483648
        let lo = seed % 23
        let hi = lo + seed / 23 % (24 - lo)
        let x = seed / 1000 % 15 - 7
        f.add(V.cuts.nth(lo)..V.cuts.nth(hi), x)
        for j in lo..hi: a[j] += x
        let i = seed / 100000 % 23
        assert f.get(V(i)) == a[i]

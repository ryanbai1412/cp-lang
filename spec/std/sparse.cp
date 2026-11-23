//! Sparse table: O(1) range folds of a fixed array.
use std.math.{gcd}
use std.seq.{ends}

/// `SparseTable<V, Int>(min, a)`: built in O(n log n) time and memory, `fold` is O(1).
/// The operation must be associative and idempotent (`op(a, a) == a`): `min`, `max`, `gcd`,
/// `&`, `|`, or any custom `(T, T) -> T` — no identity is required because `fold` asserts
/// the range is non-empty and returns `T` directly. Sums are not idempotent; use `Fenwick` or `prefix` for those.
pub struct SparseTable<D, T>(op: (T, T) -> T):
    n: Int
    table: List<T>

    /// Build a sparse table for the idempotent, associative `op` over the values of `a`. Level `k`
    /// holds the fold of every window of length `2^k`, so any non-empty range is the fold of two
    /// overlapping windows. The table copies `a` and keeps its domain `D`.
    ///
    /// cost: O(|D| log |D|) time and memory, `op` called O(|D| log |D|) times
    /// empty: an empty `a` builds an empty table; every `fold` on it traps, `try_fold` is `None`
    /// order: the table keeps the domain `D` of `a`
    /// traps: never (a non-idempotent or non-associative `op` gives unspecified folds; check with `idempotent`)
    /// example: `SparseTable(min, [3, 1, 4]).fold(0..3) == 1`
    pub fn new(op: (T, T) -> T, a: [D]T) -> SparseTable<D, T>:
        let n = D.K
        if n == 0: return SparseTable(op, 0, [])
        let levels = bit_width(n)
        var table = List.filled(levels * n, a[D.first!])
        var idx = 0
        for p in D:
            unchecked: table[idx] = a[p]
            idx += 1
        var k = 1
        var w = 1
        while k < levels:
            let prev_off = (k - 1) * n
            let off = k * n
            let lim = n - 2 * w + 1
            var i = 0
            while i < lim:
                unchecked: table[off + i] = op(table[prev_off + i], table[prev_off + i + w])
                i += 1
            w *= 2
            k += 1
        SparseTable(op, n, table)

    /// The fold of `op` over the positions of the non-empty range `r`, as two overlapping
    /// power-of-two windows combined with one `op` call.
    ///
    /// cost: O(1)
    /// empty: traps; use `try_fold` when `r` may be empty
    /// traps: when `r` is empty or reaches outside `D`
    /// example: `SparseTable(max, [3, 1, 4]).fold(0..2) == 3`
    /// example: `SparseTable(min, [3, 1, 4]).fold(1..2) == 1`
    pub fn fold(self, r: Range<D>) -> T:
        let lo = r.l.ord
        let hi = r.r.ord
        assert lo < hi
        let k = bit_width(hi - lo) - 1
        let off = k * n
        unchecked: return op(table[off + lo], table[off + hi - (1 << k)])

    /// The fold of `r` like `fold`, or `None` when `r` is empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `SparseTable(min, [3, 1, 4]).try_fold(1..1) == None`
    /// example: `SparseTable(min, [3, 1, 4]).try_fold(1..3) == Some(1)`
    pub fn try_fold(self, r: Range<D>) -> ?T:
        if r.is_empty: return None
        Some(self.fold(r))

    /// The original value at position `i` (level 0 of the table).
    ///
    /// cost: O(1)
    /// traps: when `i` is outside `D`
    /// example: `SparseTable(min, [3, 1, 4]).get(2) == 4`
    pub fn get(self, i: D) -> T:
        unchecked: return table[i.ord]

    /// The fold of every position of `D`; `fold(D.all)`.
    ///
    /// cost: O(1)
    /// empty: traps on an empty domain
    /// traps: when `D` is empty
    /// example: `SparseTable(gcd, [4, 6, 10]).all == 2`
    @get pub fn all(self) -> T:
        assert n > 0
        let k = bit_width(n) - 1
        let off = k * n
        unchecked: return op(table[off], table[off + n - (1 << k)])

/// Whether `op(x, x) == x` for every sample `x` in `xs`: the law a sparse table needs on top of
/// associativity. A sample check, not a proof; associativity is not tested.
///
/// cost: O(n) calls of `op`
/// empty: `true`
/// order: samples are checked in list order; the first failure decides
/// traps: never (`op` may trap)
/// example: `idempotent(max, [1, 5]) == true`
/// example: `idempotent((a, b) => a + b, [0, 1]) == false`
pub fn idempotent<T>(op: (T, T) -> T, xs: List<T>) -> Bool:
    for x in xs:
        if op(x, x) != x: return false
    true

/// Disjoint sparse table: O(n log n) build and memory, O(1) range fold for ANY associative
/// operation — no idempotence and no inverse required (matrix products, `Mod<P>` products,
/// `MaxSub`, string concatenation). Asserts `not r.is_empty`.
pub struct DisjointSparseTable<D, T>(op: (T, T) -> T):
    n: Int
    table: List<T>

    /// Build a disjoint sparse table for the associative `op` over `a`. Each level splits the domain
    /// into blocks and stores folds outward from each block's middle, so a range is answered by
    /// combining one suffix fold and one prefix fold; no idempotence, identity or inverse is needed.
    ///
    /// cost: O(|D| log |D|) time and memory, `op` called O(|D| log |D|) times
    /// empty: an empty `a` builds an empty table; every `fold` on it traps, `try_fold` is `None`
    /// order: the table keeps the domain `D` of `a`; folds run in domain order
    /// traps: never (a non-associative `op` gives unspecified folds)
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).fold(0..3) == 8`
    pub fn new(op: (T, T) -> T, a: [D]T) -> DisjointSparseTable<D, T>:
        let n = D.K
        if n == 0: return DisjointSparseTable(op, 0, [])
        let levels = if n <= 1: 1 else: bit_width(n - 1)
        var table = List.filled(levels * n, a[D.first!])
        var idx = 0
        for p in D:
            unchecked: table[idx] = a[p]
            idx += 1
        var k = 1
        var half = 2
        while k < levels:
            let off = k * n
            var mid = half
            while mid < n:
                unchecked:
                    table[off + mid - 1] = table[mid - 1]
                    var i = mid - 2
                    let lo_lim = mid - half
                    while i >= lo_lim:
                        table[off + i] = op(table[i], table[off + i + 1])
                        i -= 1
                    table[off + mid] = table[mid]
                    var j = mid + 1
                    let hi = min(mid + half, n)
                    while j < hi:
                        table[off + j] = op(table[off + j - 1], table[j])
                        j += 1
                mid += 2 * half
            half *= 2
            k += 1
        DisjointSparseTable(op, n, table)

    /// The fold of `op` over the non-empty range `r`, left to right in domain order, so a
    /// non-commutative `op` sees its operands in position order.
    ///
    /// cost: O(1), at most one `op` call
    /// empty: traps; use `try_fold` when `r` may be empty
    /// traps: when `r` is empty or reaches outside `D`
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).fold(1..3) == 5`
    /// example: `DisjointSparseTable((a, b) => a, [7, 8, 9]).fold(1..3) == 8`
    pub fn fold(self, r: Range<D>) -> T:
        let lo = r.l.ord
        let hi = r.r.ord
        assert lo < hi
        let r1 = hi - 1
        if lo == r1:
            unchecked: return table[lo]
        let k = bit_width(lo ^ r1) - 1
        let off = k * n
        unchecked: return op(table[off + lo], table[off + r1])

    /// The fold of `r` like `fold`, or `None` when `r` is empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).try_fold(2..2) == None`
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).try_fold(0..2) == Some(4)`
    pub fn try_fold(self, r: Range<D>) -> ?T:
        if r.is_empty: return None
        Some(self.fold(r))

    /// The original value at position `i`.
    ///
    /// cost: O(1)
    /// traps: when `i` is outside `D`
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).get(0) == 3`
    pub fn get(self, i: D) -> T:
        unchecked: return table[i.ord]

    /// The fold of every position of `D`, left to right; `fold(D.all)`.
    ///
    /// cost: O(1)
    /// empty: traps on an empty domain
    /// traps: when `D` is empty
    /// example: `DisjointSparseTable((a, b) => a + b, [3, 1, 4]).all == 8`
    @get pub fn all(self) -> T:
        assert n > 0
        let r1 = n - 1
        if r1 == 0:
            unchecked: return table[0]
        let k = bit_width(r1) - 1
        let off = k * n
        unchecked: return op(table[off], table[off + r1])

/// O(|R| * |C| * log |R| * log |C|) build and memory; O(1) rectangle fold for any associative,
/// idempotent operation (`min`, `max`, `gcd`, `&`, `|`).
pub struct SparseTable2D<R, C, T>(op: (T, T) -> T):
    table: List<T>
    h: Int
    w: Int
    lr: Int
    lc: Int

    fn at(self, kr: Int, kc: Int, i: Int, j: Int) -> Int = ((kr * lc + kc) * h + i) * w + j

    /// Build a 2-D sparse table for the idempotent, associative `op` over the grid `a`: one
    /// `SparseTable` level per power-of-two row height and column width. Keeps the domains `R`, `C`.
    ///
    /// cost: O(|R| |C| log |R| log |C|) time and memory
    /// empty: an empty grid builds an empty table; every `fold` on it traps
    /// order: the table keeps the domains `R` and `C` of `a`
    /// traps: never (a non-idempotent `op` gives unspecified folds)
    /// example: `SparseTable2D(max, [2, 2] (r, c) => r * 2 + c).fold(0..2, 0..2) == 3`
    pub fn new(op: (T, T) -> T, a: [R, C]T) -> SparseTable2D<R, C, T>:
        let (h, w) = (R.K, C.K)
        if h == 0 or w == 0: return SparseTable2D(op, [], h, w, 0, 0)
        let (lr, lc) = (bit_width(h), bit_width(w))
        var s = SparseTable2D(op, List.filled(lr * lc * h * w, a[R.first!, C.first!]), h, w, lr, lc)
        for r in R:
            for c in C: s.table[r.to_idx * w + c.to_idx] = a[r, c]
        for kr in 0..lr:
            for kc in 0..lc:
                if kr == 0 and kc == 0: continue
                for i in 0..h - (1 << kr) + 1:
                    for j in 0..w - (1 << kc) + 1:
                        s.table[s.at(kr, kc, i, j)] = if kr == 0:
                            op(s.table[s.at(0, kc - 1, i, j)], s.table[s.at(0, kc - 1, i, j + (1 << (kc - 1)))])
                        else:
                            op(s.table[s.at(kr - 1, kc, i, j)], s.table[s.at(kr - 1, kc, i + (1 << (kr - 1)), j)])
        s

    /// The fold of `op` over the rectangle `rows` by `cols`, both non-empty: four overlapping
    /// power-of-two blocks combined with `op`.
    ///
    /// cost: O(1)
    /// empty: traps when either range is empty; use `try_fold`
    /// traps: when either range is empty or reaches outside its domain
    /// example: `SparseTable2D(min, [2, 2] (r, c) => r * 2 + c).fold(1..2, 0..2) == 2`
    pub fn fold(self, rows: Range<R>, cols: Range<C>) -> T:
        let (r0, r1) = rows.ends
        let (c0, c1) = cols.ends
        assert r0 < r1 and c0 < c1, "`fold` needs a non-empty rectangle"
        let (kr, kc) = (bit_width(r1 - r0) - 1, bit_width(c1 - c0) - 1)
        let (i2, j2) = (r1 - (1 << kr), c1 - (1 << kc))
        op(op(table[self.at(kr, kc, r0, c0)], table[self.at(kr, kc, r0, j2)]), op(table[self.at(kr, kc, i2, c0)], table[self.at(kr, kc, i2, j2)]))

    /// The rectangle fold like `fold`, or `None` when `rows` or `cols` is empty.
    ///
    /// cost: O(1)
    /// empty: `None` when either range is empty
    /// traps: when a range reaches outside its domain on a non-empty rectangle
    /// example: `SparseTable2D(min, [2, 2] (r, c) => r * 2 + c).try_fold(0..2, 1..1) == None`
    /// example: `SparseTable2D(min, [2, 2] (r, c) => r * 2 + c).try_fold(0..2, 1..2) == Some(1)`
    pub fn try_fold(self, rows: Range<R>, cols: Range<C>) -> ?T:
        let (r0, r1) = rows.ends
        let (c0, c1) = cols.ends
        if r0 >= r1 or c0 >= c1: None else: Some(self.fold(rows, cols))

    /// The original cell value `a[r, c]`.
    ///
    /// cost: O(1)
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `SparseTable2D(min, [2, 2] (r, c) => r * 2 + c).get(1, 0) == 2`
    pub fn get(self, r: R, c: C) -> T:
        table[r.to_idx * w + c.to_idx]

/// O(|D|) build and memory; O(1) range min/max query via block-of-64 bitmask RMQ (Four Russians).
/// Useful for O(1) LCA and memory-tight static RMQ where O(n log n) space is too large.
pub struct LinearRmq<D, T>(before: (T, T) -> Bool):
    n: Int
    vals: List<T>
    mask: List<U64>
    nb: Int
    top: List<Int>

    fn better(self, i: Int, j: Int) -> Int = if before(vals[j], vals[i]): j else: i

    fn in_block(self, l: Int, r: Int) -> Int:
        let base = l - l % 64
        let m = mask[r] & ~((U64(1) << (l - base)) - U64(1))
        base + bit_width(m & (~m + U64(1))) - 1

    /// Build a linear-space range-best structure over `a`, where `before(x, y)` means `x` is better
    /// than `y` (default `<`, so the best is the minimum). Blocks of 64 positions carry a sparse
    /// table over block bests and an in-block bitmask stack, so queries never scan.
    ///
    /// cost: O(|D|) time and memory
    /// empty: an empty `a` builds an empty structure; `argbest` and `best` on it trap
    /// order: the structure keeps the domain `D` of `a`
    /// traps: never (`before` must be a strict weak order, or results are unspecified)
    /// example: `LinearRmq([3, 1, 4]).best(0..3) == 1`
    pub fn new(a: [D]T, before: (T, T) -> Bool = (x, y) => x < y) -> LinearRmq<D, T>:
        let vals = [a[i] for i in D]
        let n = vals.len
        let nb = (n + 63) / 64
        var q = LinearRmq<D, T>(before, n, vals, [], nb, [])
        for b in 0..nb:
            var cur: U64 = U64(0)
            for i in 64 * b..min(64 * b + 64, n):
                while cur != U64(0) and before(q.vals[i], q.vals[64 * b + bit_width(cur) - 1]):
                    cur ^= U64(1) << (bit_width(cur) - 1)
                cur |= U64(1) << (i - 64 * b)
                q.mask.push(cur)
        for b in 0..nb: q.top.push(q.in_block(64 * b, min(64 * b + 64, n) - 1))
        var k = 1
        while (1 << k) <= nb:
            for b in 0..nb:
                q.top.push(if b + (1 << k) <= nb: q.better(q.top[(k - 1) * nb + b], q.top[(k - 1) * nb + b + (1 << (k - 1))]) else: q.top[(k - 1) * nb + b])
            k += 1
        q

    /// The position of the best element of the non-empty range `r` under `before`; the leftmost
    /// when several are equally best (neither is `before` the other).
    ///
    /// cost: O(1)
    /// empty: traps
    /// ties: the first in key order
    /// traps: when `r` is empty or reaches outside `D`
    /// example: `LinearRmq([3, 1, 1, 4]).argbest(0..4) == 1`
    /// example: `LinearRmq([3, 1, 4], before: (x, y) => x > y).argbest(0..3) == 2`
    pub fn argbest(self, r: Range<D>) -> D:
        let (lo, hi) = r.ends
        assert lo < hi, "`argbest` needs a non-empty range"
        let h = hi - 1
        let (bl, bh) = (lo / 64, h / 64)
        if bl == bh: return D.nth(self.in_block(lo, h))
        var ans = self.in_block(lo, 64 * bl + 63)
        if bl + 1 < bh:
            let k = bit_width(bh - bl - 1) - 1
            ans = self.better(ans, self.better(top[k * nb + bl + 1], top[k * nb + bh - (1 << k)]))
        D.nth(self.better(ans, self.in_block(64 * bh, h)))

    /// The value of the best element of the non-empty range `r` under `before`; `a[argbest(r)]`.
    ///
    /// cost: O(1)
    /// empty: traps; use `try_best` when `r` may be empty
    /// ties: equal-best values are interchangeable; the value at the leftmost is returned
    /// traps: when `r` is empty or reaches outside `D`
    /// example: `LinearRmq([3, 1, 4], before: (x, y) => x > y).best(0..3) == 4`
    pub fn best(self, r: Range<D>) -> T:
        vals[self.argbest(r).to_idx]

    /// The value of the best element of `r` like `best`, or `None` when `r` is empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: the value at the leftmost equal-best position
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `LinearRmq([3, 1]).try_best(1..1) == None`
    /// example: `LinearRmq([3, 1]).try_best(0..2) == Some(1)`
    pub fn try_best(self, r: Range<D>) -> ?T:
        let (lo, hi) = r.ends
        if lo >= hi: None else: Some(self.best(r))

@test
fn range_min_matches_brute_force():
    domain V = 1..=13
    let a = [V] v => rand(-50, 50)
    let st = SparseTable<V, Int>(min, a)
    for l in V:
        for r in l..=V.last!:
            assert st.fold(l..=r) == [a[v] for v in l..=r].min()!
    assert st.fold(..=V(5)) == [a[v] for v in V if v <= V(5)].min()!
    assert st.fold(V(8)..) == [a[v] for v in V if v >= V(8)].min()!

@test
fn gcd_max_bitwise_and_custom_semigroup_tables():
    domain V = 0..6
    let a = [V] v => [12, 18, 6, 30, 7, 9][v.to_idx]
    assert SparseTable<V, Int>(gcd, a).fold(V(0)..=V(3)) == 6
    assert SparseTable<V, Int>(max, a).fold(V(2)..=V(5)) == 30
    assert SparseTable<V, Int>(&, a).fold(V(0)..=V(3)) == (12 & 18 & 6 & 30)
    assert SparseTable<V, Int>(|, a).fold(V(4)..=V(5)) == (7 | 9)
    // An idempotent semigroup with no identity at all: pick the entry with smaller key, tie-breaking by value
    let pairs = [V] v => (a[v], v.to_idx)
    let pick = SparseTable<V, (Int, Int)>((x, y) => if x < y: x else: y, pairs)
    assert pick.fold(V(0)..=V(3)) == (6, 2)
    assert pick.fold(V(3)..=V(3)) == (30, 3)

@test
fn idempotence_separates_min_from_sum():
    assert idempotent<Int>(min, [-2, 0, 7])
    assert idempotent<Int>(&, [-2, 0, 7])
    assert not idempotent<Int>(+, [-2, 0, 7])

@test
fn disjoint_sparse_table_handles_non_idempotent_non_commutative_folds():
    domain V = 1..=13
    let a = [V] _ => rand(-20, 20)
    let st_sum = DisjointSparseTable<V, Int>(+, a)
    for l in V:
        for r in l..=V.last!:
            assert st_sum.fold(l..=r) == sum(a[v] for v in l..=r)
    domain P = 0..7
    let words = [P] p => ["a", "b", "c", "d", "e", "f", "g"][p.to_idx]
    let st_cat = DisjointSparseTable<P, Str>((x, y) => x + y, words)
    assert st_cat.fold(P(0)..=P(6)) == "abcdefg"
    assert st_cat.fold(P(2)..=P(4)) == "cde"
    assert st_cat.fold(P(5)..=P(5)) == "f"

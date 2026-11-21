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

    /// Build an idempotent range-query sparse table for `op` over `a`. O(|D| log |D|). Law: `op(x, x) == x`.
    pub fn new(op: (T, T) -> T, a: [D]T) -> SparseTable<D, T>:
        let n = D.K
        if n == 0: return SparseTable(op, 0, [])
        let levels = bit_width(n)
        var table = List.filled(levels * n, a[D.first])
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

    /// The fold of a non-empty range. O(1). Asserts `not r.is_empty`.
    pub fn fold(self, r: Range<D>) -> T:
        let lo = r.lo.ord
        let hi = r.hi.ord
        assert lo < hi
        let k = bit_width(hi - lo) - 1
        let off = k * n
        unchecked: return op(table[off + lo], table[off + hi - (1 << k)])

/// Is `op` idempotent on these samples? The law a sparse table needs alongside associativity.
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

    /// Build an associative O(1)-query disjoint sparse table for `op` over `a`. O(|D| log |D|).
    pub fn new(op: (T, T) -> T, a: [D]T) -> DisjointSparseTable<D, T>:
        let n = D.K
        if n == 0: return DisjointSparseTable(op, 0, [])
        let levels = if n <= 1: 1 else: bit_width(n - 1)
        var table = List.filled(levels * n, a[D.first])
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

    /// Fold of a non-empty range `r`, left to right. O(1). Asserts `not r.is_empty`.
    pub fn fold(self, r: Range<D>) -> T:
        let lo = r.lo.ord
        let hi = r.hi.ord
        assert lo < hi
        let r1 = hi - 1
        if lo == r1:
            unchecked: return table[lo]
        let k = bit_width(lo ^ r1) - 1
        let off = k * n
        unchecked: return op(table[off + lo], table[off + r1])

@test
fn range_min_matches_brute_force():
    domain V = 1..=13
    let a = [V] v => rand(-50, 50)
    let st = SparseTable<V, Int>(min, a)
    for l in V:
        for r in l..=V.last:
            assert st.fold(l..=r) == min(a[v] for v in l..=r)!
    assert st.fold(..=V(5)) == min(a[v] for v in V if v <= V(5))!
    assert st.fold(V(8)..) == min(a[v] for v in V if v >= V(8))!

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
    assert idempotent::<Int>(min, [-2, 0, 7])
    assert idempotent::<Int>(&, [-2, 0, 7])
    assert not idempotent::<Int>(+, [-2, 0, 7])

@test
fn disjoint_sparse_table_handles_non_idempotent_non_commutative_folds():
    domain V = 1..=13
    let a = [V] _ => rand(-20, 20)
    let st_sum = DisjointSparseTable<V, Int>(+, a)
    for l in V:
        for r in l..=V.last:
            assert st_sum.fold(l..=r) == sum(a[v] for v in l..=r)
    domain P = 0..7
    let words = [P] p => ["a", "b", "c", "d", "e", "f", "g"][p.to_idx]
    let st_cat = DisjointSparseTable<P, Str>((x, y) => x + y, words)
    assert st_cat.fold(P(0)..=P(6)) == "abcdefg"
    assert st_cat.fold(P(2)..=P(4)) == "cde"
    assert st_cat.fold(P(5)..=P(5)) == "f"

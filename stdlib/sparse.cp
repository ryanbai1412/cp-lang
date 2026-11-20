//! Sparse table: O(1) range folds of a fixed array.
use std.algebra.{Semigroup}

/// Built in O(n log n) time and memory; `fold` is O(1).
/// The operation must be idempotent (`op(a, a) == a`): min, max, gcd, and, or.
/// Sums are not; use `Fenwick` or a prefix array for those.
pub struct SparseTable<D, T>(s: Semigroup<T>):
    rows: List<List<T>>

    pub fn new(s: Semigroup<T>, a: [D]T) -> SparseTable<D, T>:
        var rows = [[a[i] for i in D]]
        var w = 1
        while 2 * w <= D.K:
            let prev = rows.last
            rows.push([s.op(prev[i], prev[i + w]) for i in 0..=D.K - 2 * w])
            w *= 2
        SparseTable(s, rows)

    /// The fold of `a[l..=r]`; `none` for an empty range.
    pub fn fold(self, r: Range<D>) -> ?T:
        if r.is_empty: return none
        let k = bit_width(r.len) - 1
        s.op(rows[k][r.lo.ord], rows[k][r.hi.ord - (1 << k)])

/// Is `op` idempotent on these samples? The law a sparse table adds to `Semigroup`.
pub fn idempotent<T>(s: Semigroup<T>, xs: List<T>) -> Bool:
    for x in xs:
        if s.op(x, x) != x: return false
    true

@test
fn range_min_matches_brute_force():
    domain V = 1..=13
    let a = [V] v => rand(-50, 50)
    let lo = Semigroup<Int>((x, y) => min(x, y))
    let st = SparseTable<V, Int>(lo, a)
    for l in V:
        for r in l..=V.last:
            assert st.fold(l..=r)! == min(a[v] for v in l..=r)
    assert st.fold(V(2)..V(2)) == none

@test
fn idempotence_separates_min_from_sum():
    assert idempotent(Semigroup<Int>((x, y) => min(x, y)), [-2, 0, 7])
    assert not idempotent(Semigroup<Int>((x, y) => x + y), [-2, 0, 7])

//! Sparse table: O(1) range folds of a fixed array.
use std.algebra.{Monoid}
use std.math.{gcd}

/// `SparseTable<V, Int>(min, a)`: built in O(n log n) time and memory, `fold` is O(1).
/// The operation must be idempotent (`op(a, a) == a`): `min`, `max`, `gcd`, `&`, `|`.
/// Sums are not; use `Fenwick` or `prefix` for those.
pub struct SparseTable<D, T>(m: Monoid<T>):
    rows: List<List<T>>

    pub fn new<M>(op: M, a: [D]T) -> SparseTable<D, T>:
        let m = Monoid<T>.of(op)
        var rows = [[a[i] for i in D]]
        var w = 1
        while 2 * w <= D.K:
            let prev = rows[rows.len - 1]
            rows.push([m.op(prev[i], prev[i + w]) for i in 0..=D.K - 2 * w])
            w *= 2
        SparseTable(m, rows)

    /// The fold of a range; the identity for an empty one.
    pub fn fold(self, r: Range<D>) -> T:
        if r.is_empty: return m.id
        let k = bit_width(r.len) - 1
        m.op(rows[k][r.lo.ord], rows[k][r.hi.ord - (1 << k)])

/// Is `op` idempotent on these samples? The law a sparse table needs on top of a monoid's.
pub fn idempotent<T, M>(op: M, xs: List<T>) -> Bool:
    let m = Monoid<T>.of(op)
    for x in xs:
        if m.op(x, x) != x: return false
    true

@test
fn range_min_matches_brute_force():
    domain V = 1..=13
    let a = [V] v => rand(-50, 50)
    let st = SparseTable<V, Int>(min, a)
    for l in V:
        for r in l..=V.last:
            assert st.fold(l..=r) == min(a[v] for v in l..=r)
    assert st.fold(V(2)..V(2)) == Int.MAX
    assert st.fold(..=V(5)) == min(a[v] for v in V if v <= V(5))

@test
fn gcd_and_max_tables():
    domain V = 0..6
    let a = [V] v => [12, 18, 6, 30, 7, 9][v.to_idx]
    assert SparseTable<V, Int>(gcd, a).fold(V(0)..=V(3)) == 6
    assert SparseTable<V, Int>(max, a).fold(V(2)..=V(5)) == 30

@test
fn idempotence_separates_min_from_sum():
    assert idempotent::<Int>(min, [-2, 0, 7])
    assert not idempotent::<Int>(+, [-2, 0, 7])

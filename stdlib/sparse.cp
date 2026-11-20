//! Sparse table: O(1) range folds of a fixed array.
use std.math.{gcd}
use std.seq.{ends}

/// `SparseTable<V, Int>(min, a)`: built in O(n log n) time and memory, `fold` is O(1).
/// The operation must be associative and idempotent (`op(a, a) == a`): `min`, `max`, `gcd`,
/// `&`, `|`, or any custom `(T, T) -> T` — no identity is required because `fold` asserts
/// the range is non-empty and returns `T` directly. Sums are not idempotent; use `Fenwick` or `prefix` for those.
pub struct SparseTable<D, T>(op: (T, T) -> T):
    rows: List<List<T>>

    pub fn new(op: (T, T) -> T, a: [D]T) -> SparseTable<D, T>:
        var rows = [[a[i] for i in D]]
        var w = 1
        while 2 * w <= D.K:
            let prev = rows[rows.len - 1]
            rows.push([op(prev[i], prev[i + w]) for i in 0..=D.K - 2 * w])
            w *= 2
        SparseTable(op, rows)

    /// The fold of a non-empty range. O(1). Asserts `not r.is_empty`.
    pub fn fold(self, r: Range<D>) -> T:
        let (lo, hi) = r.ends
        assert lo < hi
        let k = bit_width(hi - lo) - 1
        op(rows[k][lo], rows[k][hi - (1 << k)])

/// Is `op` idempotent on these samples? The law a sparse table needs alongside associativity.
pub fn idempotent<T>(op: (T, T) -> T, xs: List<T>) -> Bool:
    for x in xs:
        if op(x, x) != x: return false
    true

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

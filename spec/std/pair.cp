//! Unordered pairs of distinct positions: the domain of data given once per pair.

/// An unordered pair `{lo, hi}` of distinct positions of `V`, `lo` before `hi`. `Pair<V>` is a
/// domain of `V.count * (V.count - 1) / 2` positions, so `[Pair<V>]T` stores one value per pair,
/// packed, with no diagonal and no mirrored half. It walks by `hi`, then by `lo`: `(0, 1)`,
/// `(0, 2)`, `(1, 2)`, `(0, 3)`, ..., which is the order a lower triangle is given in (row `i`
/// lists the pairs with every earlier `j`), so `read<[Pair<V>]Int>()` reads one. An array over it
/// takes two positions in either order: `d[u, v] == d[v, u]`, trapping when `u == v`.
pub struct Pair<V>(lo: V, hi: V):
    /// The pair of `u` and `v`, in either order. Traps when `u == v`. O(1).
    pub fn new(u: V, v: V) -> Pair<V> = todo

    /// `None` when `u == v`, else `Pair(u, v)`. O(1).
    pub fn at(u: V, v: V) -> ?Pair<V> = todo

    /// The pair's other end: `p.other(p.lo) == p.hi`. Traps when `v` is not in `p`. O(1).
    pub fn other(self, v: V) -> V = todo

/// The full matrix of a per-pair table: `m[u, v] == m[v, u] == d[u, v]`, and `diag` on the
/// diagonal. `read<[Pair<V>]Int>().map(Some).symmetric(diag: None)` is a distance matrix for
/// `floyd_warshall`. O(n²).
pub fn symmetric<V, T>(self: [Pair<V>]T, diag: T) -> [V, V]T = todo

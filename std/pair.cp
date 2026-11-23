//! Unordered pairs of distinct positions: the domain of data given once per pair.

/// An unordered pair `{lo, hi}` of distinct positions of `V`, `lo` before `hi`. `Pair<V>` is a
/// domain of `V.count * (V.count - 1) / 2` positions, so `[Pair<V>]T` stores one value per pair,
/// packed, with no diagonal and no mirrored half. It walks by `hi`, then by `lo`: `(0, 1)`,
/// `(0, 2)`, `(1, 2)`, `(0, 3)`, ..., which is the order a lower triangle is given in (row `i`
/// lists the pairs with every earlier `j`), so `read<[Pair<V>]Int>()` reads one. An array over it
/// takes two positions in either order: `d[u, v] == d[v, u]`, trapping when `u == v`.
pub struct Pair<V>(pub lo: V, pub hi: V):
    /// The pair of `u` and `v`, in either order: the smaller position becomes `lo`.
    ///
    /// cost: O(1)
    /// traps: when `u == v`
    /// example: `Pair<0..4>(3, 1).lo == 1`
    /// example: `Pair<0..4>(3, 1) == Pair<0..4>(1, 3)`
    pub fn new(u: V, v: V) -> Pair<V>:
        assert u != v, "`Pair` needs two distinct positions"
        if u < v: Pair(u, v) else: Pair(v, u)

    /// The pair of `u` and `v` in either order, or `None` when `u == v`: the non-trapping
    /// [`new`](Pair.new).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Pair<0..4>.at(2, 2) == None`
    /// example: `Pair<0..4>.at(2, 0) == Some(Pair<0..4>(0, 2))`
    pub fn at(u: V, v: V) -> ?Pair<V> = if u == v: None else: Some(Pair.new(u, v))

    /// The end of the pair that is not `v`: `p.other(p.lo) == p.hi` and `p.other(p.hi) == p.lo`.
    ///
    /// cost: O(1)
    /// traps: when `v` is neither `self.lo` nor `self.hi`
    /// example: `Pair<0..4>(1, 3).other(3) == 1`
    pub fn other(self, v: V) -> V:
        assert v == lo or v == hi, "`Pair.other` needs one of the pair's ends"
        if v == lo: hi else: lo

// blocked: `[Pair<V>]T` is rejected as an array key (GAPS.md pair)
/// The full square matrix of a per-pair table: `m[u, v] == m[v, u] == self[u, v]` for
/// `u != v`, and `diag` at every `m[v, v]`. `read<[Pair<V>]Int>().map(Some).symmetric(diag: None)`
/// is a distance matrix for `floyd_warshall`.
///
/// cost: O(n²) for `n == V.count`; allocates the matrix
/// empty: the empty matrix when `V` is empty; `[[diag]]` when `V` has one position
/// order: keyed by `(V, V)`, row-major
/// traps: never
/// example: `let d: [Pair<0..3>]Int = [5, 6, 7]; let m = d.symmetric(diag: 0); m[2, 1] == 7 and m[0, 2] == 6 and m[1, 1] == 0`
pub fn symmetric<V, T>(self: [Pair<V>]T, diag: T) -> [V, V]T = todo

//! Unordered pairs of distinct positions: the domain of data given once per pair.

use std.math.{isqrt}

/// An unordered pair of distinct positions `lo` and `hi` of `V`, `lo` before `hi`. `Pair<V>` is a
/// domain of `V.count * (V.count - 1) / 2` positions, so `[Pair<V>]T` stores one value per pair,
/// packed, with no diagonal and no mirrored half. It walks by `hi`, then by `lo`: `(0, 1)`,
/// `(0, 2)`, `(1, 2)`, `(0, 3)`, ..., which is the order a lower triangle is given in (row `i`
/// lists the pairs with every earlier `j`), so `read<[Pair<V>]Int>()` reads one. An array over it
/// takes two positions in either order: `d[u, v] == d[v, u]`, trapping when `u == v`.
pub struct Pair<V>(pub lo: V, pub hi: V):
    /// The pair of `u` and `v`, in either order: the smaller position becomes `lo`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `u == v`
    /// example: `Pair<0..4>(3, 1).lo == 1`
    /// example: `Pair<0..4>(3, 1) == Pair<0..4>(1, 3)`
    pub fn new(u: V, v: V) -> Pair<V>:
        assert u != v, "`Pair` needs two distinct positions"
        if u < v: Pair(u, v) else: Pair(v, u)

    /// The pair of `u` and `v` in either order, or `None` when `u == v`: the non-trapping
    /// [`new`](Pair.new).
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Pair<0..4>.at(2, 2) == None`
    /// example: `Pair<0..4>.at(2, 0) == Some(Pair<0..4>(0, 2))`
    pub fn at(u: V, v: V) -> ?Pair<V> = if u == v: None else: Some(Pair.new(u, v))

    /// The end of the pair that is not `v`: `p.other(p.lo) == p.hi` and `p.other(p.hi) == p.lo`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `v` is neither `self.lo` nor `self.hi`
    /// example: `Pair<0..4>(1, 3).other(3) == 1`
    pub fn other(self, v: V) -> V:
        assert v == lo or v == hi, "`Pair.other` needs one of the pair's ends"
        if v == lo: hi else: lo

impl Index for Pair<V>:
    let count = V.count * (V.count - 1) / 2
    @get fn ord(self) -> Int = hi.ord * (hi.ord - 1) / 2 + lo.ord
    fn nth(k: Int) -> Pair<V>:
        var h = (1 + isqrt(1 + 8 * k)) / 2
        while h * (h - 1) / 2 > k: h -= 1
        Pair(V.nth(k - h * (h - 1) / 2), V.nth(h))

impl Into<Int> for Pair<V>:
    fn into(self) -> Int = self.ord

impl Show for Pair<V>:
    fn show(a) = "{a.ord}"

/// The full square matrix of a per-pair table: `m[u, v] == m[v, u] == self[u, v]` for
/// `u != v`, and `diag` at every `m[v, v]`. `read<[Pair<V>]Int>().map(Some).symmetric(diag: None)`
/// is a distance matrix for `floyd_warshall`.
///
/// `V` with one position gives `[[diag]]`.
///
/// cost: \(O(n^2)\) for `n == V.count`; allocates the matrix
/// empty: `domain V = 0..0; let d: [Pair<V>]Int = []; d.symmetric(diag: 0).len == 0`
/// order: keyed by `(V, V)`, row-major
/// traps: never
/// example: `domain V = 0..3; let d: [Pair<V>]Int = [5, 6, 7]; let m = d.symmetric(diag: 0); m[2, 1] == 7 and m[0, 2] == 6 and m[1, 1] == 0`
pub fn symmetric<V, T>(self: [Pair<V>]T, diag: T) -> [V, V]T:
    // In 64 x 64 tiles: the mirrored write `m[l, h]` walks a column, which
    // misses the cache on every row once the matrix outgrows it.
    let n = V.count
    var m: [V, V]T = diag
    for th in 0..(n + 63) / 64:
        for tl in 0..th + 1:
            for h in th * 64..min(th * 64 + 64, n):
                for l in tl * 64..min(tl * 64 + 64, h):
                    let x = self[Pair(V.nth(l), V.nth(h))]
                    m[V.nth(h), V.nth(l)] = x
                    m[V.nth(l), V.nth(h)] = x
    m

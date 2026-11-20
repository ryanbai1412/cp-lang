//! Fenwick tree (binary indexed tree) over a domain.
use std.algebra.{Group, Sum, Xor}

/// Point update, prefix and range fold over a `Group`.
/// `add`, `prefix` and `fold` are O(log n); memory is n + 1 values.
/// It takes a `Group`, not a `Monoid`: a range is a prefix with another
/// prefix undone, so "Fenwick on max" does not type-check.
pub struct Fenwick<D, T>(g: Group<T>):
    t: List<T>

    pub fn new(g: Group<T>) -> Fenwick<D, T>:
        Fenwick(g, List.filled(D.K + 1, g.id))

    /// Build from an array in O(n).
    pub fn from(g: Group<T>, a: [D]T) -> Fenwick<D, T>:
        var t = List.filled(D.K + 1, g.id)
        for i in D:
            let p = i.to_idx + 1
            t[p] = g.op(t[p], a[i])
            let q = p + (p & -p)
            if q <= D.K: t[q] = g.op(t[q], t[p])
        Fenwick(g, t)

    /// `a[i] = op(a[i], x)`.
    pub fn add(mut self, i: D, x: T):
        var p = i.to_idx + 1
        while p < t.len:
            t[p] = g.op(t[p], x)
            p += p & -p

    /// The fold of everything left of the boundary `b`: `prefix(v.r)` includes `v`, `prefix(v.l)` does not.
    pub fn prefix(self, b: Bounds<D>) -> T:
        var p = b.ord
        var acc = g.id
        while p > 0:
            acc = g.op(acc, t[p])
            p -= p & -p
        acc

    /// The fold of `a[l..=r]`.
    pub fn fold(self, r: Range<D>) -> T = g.op(self.prefix(r.hi), g.inv(self.prefix(r.lo)))

    /// The first boundary where the prefix sum reaches `target`, for non-negative values:
    /// the smallest `b` with `prefix(b) >= target`, or `none`. O(log n).
    pub fn lower_bound(self, target: Int) -> ?Bounds<D>:
        if target <= 0: return D.bounds.nth(0)
        var p = 0
        var rest = target
        var step = bit_ceil(D.K + 1) / 2
        while step > 0:
            if p + step <= D.K and t[p + step] < rest:
                p += step
                rest -= t[p]
            step /= 2
        if p == D.K: none else: D.bounds.nth(p + 1)

@test
fn sums_match_a_plain_array():
    domain V = 1..=9
    var a = [V] v => rand(-5, 5)
    var f = Fenwick<V, Int>.from(Sum<Int>, a)
    for _ in 0..60:
        let i = V.nth(rand(0, 8))
        let x = rand(-9, 9)
        a[i] += x
        f.add(i, x)
        let l = V.nth(rand(0, 8))
        let r = V.nth(rand(l.to_idx, 8))
        assert f.fold(l..=r) == sum(a[v] for v in l..=r)
    assert f.prefix(V.first.l) == 0

@test
fn xor_group_and_empty_range():
    domain V = 0..5
    var f = Fenwick<V, Int>(Xor<Int>)
    f.add(V(1), 6)
    f.add(V(3), 5)
    assert f.fold(V(0)..=V(4)) == 3
    assert f.fold(V(2)..V(2)) == 0

@test
fn lower_bound_finds_the_kth():
    domain V = 0..8
    var f = Fenwick<V, Int>(Sum<Int>)
    for v in [1, 4, 4, 6]: f.add(V(v), 1)
    assert f.lower_bound(1)!.left == V(1)
    assert f.lower_bound(3)!.left == V(4)
    assert f.lower_bound(4)!.left == V(6)
    assert f.lower_bound(5) == none

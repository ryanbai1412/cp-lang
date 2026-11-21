// Five points in general position form a pentagram iff their convex hull has size 5.
// Every 5-subset has hull size 5, 4, or 3:
// - each (CCW triangle, interior point) pair extends to n - 4 five-subsets, counting 2*c4 + 4*c3;
// - each (CCW triangle, pair of interior points) uniquely counts a hull-3 five-subset (c3).
// So c5 = C(n, 5) - sum(inside) * (n - 4) / 2 + sum(C(c, 2)).
use std.geometry.{Pt, turn}
use std.math.choose

let n: 5..=300 = read()
domain V = 0..n
let pts: [V]Pt = read()

let left: [V, V]Bitset<V> = [V, V] (i, j) => Bitset<V>.of(
    k for k in V if turn(pts[i], pts[j], pts[k]) > 0
)
let inside = [
    (left[i, j] & left[j, k] & left[k, i]).len
    for i in V for j in V.after(i) for k in V.after(i) if k in left[i, j]
]

print(choose(n, 5) - inside.sum() * (n - 4) / 2 + [choose(c, 2) for c in inside].sum())

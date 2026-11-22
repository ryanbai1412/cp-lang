// Five points in general position form a pentagram iff they are in convex position (hull size 5).
// Every 5-subset has hull size 5, 4, or 3:
// - each (triangle, interior point) pair extends to n - 4 five-subsets, counting 2*c4 + 4*c3;
// - each (triangle, pair of interior points) uniquely counts a hull-3 five-subset (c3).
// So c5 = C(n, 5) - sum(inside) * (n - 4) / 2 + sum(C(inside, 2)).
use std.geometry.{turn}
use std.math.{choose}

let n: 5..=300 = read()
domain V = 0..n
let pts: [V]Pt = read()

let left = [V, V] (i, j) => Bitset<V>.of(k for k in V if turn(pts[i], pts[j], pts[k]) > 0)

// A triangle with c points inside adds C(c, 2) - c * (n - 4) / 2.
fn net(c: Int) -> Int = c * (c - (n - 3))
let twice = sum(
    net(
        (left[i, j] & left[j, k] & left[k, i]).len
    ) for i in V for j in V.after(i) for k in V.after(i) if k in left[i, j]
)
print(choose(n, 5) + twice / 2)

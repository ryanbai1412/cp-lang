// Five points in general position form a pentagram iff they are in convex position (hull size 5).
// Every 5-subset has hull size 5, 4, or 3:
// - each (triangle, interior point) pair extends to n - 4 five-subsets, counting 2*c4 + 4*c3;
// - each (triangle, pair of interior points) uniquely counts a hull-3 five-subset (c3).
// So c5 = C(n, 5) - interior_pairs * (n - 4) / 2 + hull3.
use std.geometry.{Pt, turn}

let n: 5..=300 = read()
domain V = 0..n
let pts: [V]Pt = read()

let left = [V, V] (i, j) => Bitset<V>.of(k for k in V if turn(pts[i], pts[j], pts[k]) > 0)

var interior_pairs = 0
var hull3 = 0
for i in V:
    for j in V.after(i):
        for k in V.after(i) if k in left[i, j]:
            let inside = (left[i, j] & left[j, k] & left[k, i]).len
            interior_pairs += inside
            hull3 += inside * (inside - 1) / 2

let total5 = n * (n - 1) * (n - 2) * (n - 3) * (n - 4) / 120
print(total5 - interior_pairs * (n - 4) / 2 + hull3)

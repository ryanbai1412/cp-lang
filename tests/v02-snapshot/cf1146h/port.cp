// Source: Codeforces 1146H (Satanic Panic), https://codeforces.com/contest/1146/problem/H
// Problem: n <= 300 points, |x|, |y| <= 1e6, no three collinear. Count the 5-point subsets
//   that form a pentagram.
// Solution: a pentagram is 5 points in convex position (hull size 5); 5-subsets with hull
//   4 or 3 are counted through triangles: c5 = C(n, 5) - sum(inside) * (n - 4) / 2 +
//   sum(C(inside, 2)) over all triangles. Bitsets of the points left of each directed pair
//   give inside = |left(ij) & left(jk) & left(ki)|, each triangle once by orientation:
//   O(n^3 * n / 64).
// Every 5-subset has hull size 5, 4, or 3:
// - each (triangle, interior point) pair extends to n - 4 five-subsets, counting 2*c4 + 4*c3;
// - each (triangle, pair of interior points) uniquely counts a hull-3 five-subset (c3).
use std.math.{choose}

let n: 5..=300 = read()
domain V = 0..n
let pts: [V]Pt = read()

let left = [V, V] (i, j) => Bitset<V>.of(k for k in V if turn(pts[i], pts[j], pts[k]) > 0)

// A triangle with `inside` points inside it adds C(inside, 2) - inside * (n - 4) / 2 to c5.
// That is a half-integer for some triangles, so sum twice the excess and halve once at the end.
fn twice_excess(inside: Int) -> Int = inside * (inside - 1) - inside * (n - 4)
let twice_total = sum(
    twice_excess(
        (left[i, j] & left[j, k] & left[k, i]).len
    ) for i in V for j in V.after(i) for k in V.after(i) if k in left[i, j]
)
print(choose(n, 5) + twice_total / 2)

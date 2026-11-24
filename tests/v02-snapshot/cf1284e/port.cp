// Source: Codeforces 1284E (New Year and Castle Construction),
//   https://codeforces.com/contest/1284/problem/E
// Problem: n <= 2500 distinct points, |x|, |y| <= 1e9, no three collinear. f(p) is the number of
//   4-subsets of the other points that form a quadrilateral strictly enclosing p. Print the sum
//   of f(p) over all p.
// Solution: a 4-subset misses p exactly when all four lie in one open half-plane through p; it
//   then has a unique first ray CCW around p, the other three within the half-turn after it. Per
//   centre, sort the other rays by angle; a cyclic two-pointer `reach` counts the k rays in the
//   half-turn after each, which starts choose(k, 3) missing subsets. Answer
//   n * choose(n - 1, 4) minus all missing ones, O(n^2 log n).
use std.math.choose

let n: 5..=2500 = read()
domain V = 0..n
let pts: [V]Pt = read()

fn missing(c: V) -> Int:
    let rays = [pts[v] - pts[c] for v in V if v != c].sorted(by: ByAngle)
    let ahead = rays.reach(while: (r, s) => cross(r, s) > 0, cyclic: true)
    sum(choose(k, 3) for k in ahead)

print(n * choose(n - 1, 4) - sum(missing(c) for c in V))

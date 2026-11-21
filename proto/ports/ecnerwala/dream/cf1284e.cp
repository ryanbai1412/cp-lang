// Codeforces 1284 E: New Year and Castle Construction.
//
// Count pairs (p, Q): Q four other points whose quadrilateral encloses p. Q
// misses p exactly when all four lie in one open half-plane through p. With
// no three points collinear, such a Q has a unique first ray in CCW order
// around p, the other three strictly within the half-turn after it. So per
// center, sort the other rays by angle: the rays within the half-turn after a
// ray are the next ones in cyclic order, `reach` counts them with two
// pointers, and a ray with k of them starts choose(k, 3) missing quadruples.
use std.math.choose

let n: 5..=2500 = read()
domain V = 0..n
let pts: [V]Pt = read()

fn missing(c: V) -> Int:
    let rays = [pts[v] - pts[c] for v in V if v != c].sorted(by: ByAngle)
    let ahead = rays.reach(while: (r, s) => r.cross(s) > 0, cyclic: true)
    sum(choose(k, 3) for k in ahead)

print(n * choose(n - 1, 4) - sum(missing(c) for c in V))

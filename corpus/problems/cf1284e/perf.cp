// Codeforces 1284 E, tuned: orig.cpp's rotating sweep in place of port.cp's n
// angular sorts. Sort the points by (x, y); every pair (i, j), i < j, turns
// the projection order at the direction pts[j] - pts[i], so sorting those
// directions once and swapping the pair at each counts, for every pair, the
// points on each side: (points after j) * (points before i) crossings. Four
// points in convex position give two such crossings fewer than one inside a
// triangle, and each interior quadruple has n - 4 fifth points, halved.
// No three points are collinear, so the swapped pair is always adjacent.
use std.math.choose

struct Dir(d: Pt)

impl Ord for Dir:
    fn lt(a, b) = a.d.cross(b.d) > 0

impl Eq for Dir:
    fn eq(a, b) = a.d.cross(b.d) == 0

let n: 5..=2500 = read()
let pts = read<[0..n]Pt>().sorted(by: p => (p.x, p.y))
var turns: List<(d: Pt, a: 0..n, b: 0..n)> = []
turns.reserve(n * (n - 1) / 2)
for i in 0..n:
    for j in i + 1..n: turns.push((pts[j] - pts[i], i, j))
turns.sort(by: t => Dir(t.d))
var at = (0..n).collect()
var crossings = 0
unchecked:
    for (_, a, b) in turns:
        crossings += (n - 1 - at[b]) * at[a]
        at.swap_idx(a, b)
print((crossings - 2 * choose(n, 4)) * (n - 4) / 2)

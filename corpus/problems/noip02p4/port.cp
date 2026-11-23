// Source: NOIP 2002 Senior P4 (Rectangle Cover), Luogu P1034 (not on DMOJ)
// Problem: n <= 50 integer points (coordinates 0..=500). Cover them with k axis-parallel
//   rectangles that are pairwise fully disjoint (no shared edge or corner); a rectangle over
//   one point or a line has area 0. Print the least total area. Statement: 1 <= k <= 4.
// Solution: with k <= 3 disjoint rectangles a vertical or horizontal line always separates
//   them, so cover(points, k) tries every cut between two distinct x (or y) coordinates and
//   every split j + (k - j) of the rectangles, recursing on both sides; k = 1 is the bounding
//   box. Brute force, fine for n <= 50. The port only takes k <= 3.
// original: corpus/submissions/5096459.cpp
// Points travel as List<(Int, Int)> and the two halves are copied out by a
// loop: the prototype has no list slices as arguments.
let n: 1..=50 = read()
let k: 1..=3 = read()
var pts: List<(Int, Int)> = []
for _ in 0..n:
    let x: Int = read()
    let y: Int = read()
    pts.push((x, y))

fn area(ps: List<(Int, Int)>) -> Int:
    if ps.len == 0: return 0
    var (xl, xr, yl, yr) = (ps[0].0, ps[0].0, ps[0].1, ps[0].1)
    for p in ps:
        xl min= p.0
        xr max= p.0
        yl min= p.1
        yr max= p.1
    (xr - xl) * (yr - yl)

// least total area of at most `k` rectangles: cut with a vertical or a
// horizontal line between two distinct coordinates and share k across the sides
fn cover(ps: List<(Int, Int)>, k: Int) -> Int:
    if k == 1 or ps.len == 0: return area(ps)
    var best = 1e18
    for turn in 0..2:
        var s: List<(Int, Int)> = [if turn == 0: p else: (p.1, p.0) for p in ps]
        s.sort()
        for i in 0..s.len:
            if i == 0 or s[i - 1].0 < s[i].0:
                var a: List<(Int, Int)> = []
                var b: List<(Int, Int)> = []
                for j in 0..s.len:
                    if j < i: a.push(s[j])
                    else: b.push(s[j])
                for j in 1..k: best min= cover(a, j) + cover(b, k - j)
    best

print(cover(pts, k))

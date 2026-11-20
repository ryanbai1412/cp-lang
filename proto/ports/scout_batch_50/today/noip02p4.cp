// NOIP 2002 P4, cover n points with at most k <= 3 disjoint axis-parallel rectangles (buildable today-form)
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
        var s: List<(Int, Int)> = []
        for p in ps: s.push(if turn == 0: p else: (p.1, p.0))
        sort(mut s)
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

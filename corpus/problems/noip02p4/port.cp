// Source: NOIP 2002 Senior P4 (Rectangle Cover), Luogu P1034 (not on DMOJ)
// Problem: 1 <= n <= 50 integer points (coordinates 0..=500). Cover them with 1 <= k <= 4
//   axis-parallel rectangles that are pairwise fully disjoint (no shared edge or corner); a
//   rectangle over one point or a line has area 0. Print the least total area.
// Solution: disjoint rectangles are either split by a vertical or horizontal line, or (only
//   for four) form a pinwheel round an empty hole. cover(points, k) tries every cut between
//   two distinct x (or y) coordinates and every split j + (k - j), recursing on both sides;
//   k = 1 is the bounding box. For k = 4 `pinwheel` also tries every hole x1 < x <= x2,
//   y1 < y <= y2 at point coordinates, in both turning directions, and sends each point to
//   its arm. O(n^5) for the pinwheels, fine for n <= 50.
// original: corpus/submissions/5096459.cpp (k <= 3)
let n: 1..=50 = read()
let k: 1..=4 = read()
var pts: List<(Int, Int)> = []
for _ in 0..n:
    let (x, y): (0..=500, 0..=500) = read()
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

// four arms round the hole x1 < x <= x2, y1 < y <= y2 (no point inside): the top arm takes
// x <= x2, the right one x > x2 down to y1, the bottom one x > x1, the left one the rest
fn pinwheel(ps: List<(Int, Int)>) -> Int:
    var xs = [p.0 for p in ps]
    var ys = [p.1 for p in ps]
    xs.sort()
    ys.sort()
    xs.dedup()
    ys.dedup()
    var best = 1e18
    for x1 in xs:
        for x2 in xs:
            if x2 <= x1: continue
            for y1 in ys:
                for y2 in ys:
                    if y2 <= y1: continue
                    // each arm's bounding box, empty while lo > hi
                    var xl: [4]Int = 1e9
                    var yl: [4]Int = 1e9
                    var xr: [4]Int = -1e9
                    var yr: [4]Int = -1e9
                    var hole = false
                    for (x, y) in ps:
                        let arm = if y > y2: (if x <= x2: 0 else: 1)
                            elif y > y1: (if x > x2: 1 elif x <= x1: 3 else: 4)
                            else: (if x > x1: 2 else: 3)
                        if arm == 4:
                            hole = true
                            break
                        xl[arm] min= x
                        xr[arm] max= x
                        yl[arm] min= y
                        yr[arm] max= y
                    if not hole: best min= sum((xr[a] - xl[a]) * (yr[a] - yl[a]) for a in 0..4 if xl[a] <= xr[a])
    best

// least total area of at most `k` rectangles
fn cover(ps: List<(Int, Int)>, k: Int) -> Int:
    if k == 1 or ps.len == 0: return area(ps)
    var best = 1e18
    for turn in 0..2:
        var s: List<(Int, Int)> = [if turn == 0: p else: (p.1, p.0) for p in ps]
        s.sort()
        for i in 0..s.len:
            if i == 0 or s[i - 1].0 < s[i].0:
                let (a, b) = (s[0..i].to_list(), s[i..s.len].to_list())
                for j in 1..k: best min= cover(a, j) + cover(b, k - j)
    if k == 4:
        best min= pinwheel(ps)
        best min= pinwheel([(-p.0, p.1) for p in ps])  // the other turning direction
    best

print(cover(pts, k))

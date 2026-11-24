// Source: NOIP 2002 Senior P4 (Rectangle Cover), Luogu P1034 (not on DMOJ)
// limits: 1000 ms, 125 MB
// Problem: 1 <= n <= 50 integer points (coordinates 0..=500). Cover them with 1 <= k <= 4
//   axis-parallel rectangles that are pairwise fully disjoint (no shared edge or corner); a
//   rectangle over one point or a line has area 0. Print the least total area.
// Solution: disjoint rectangles are either split by a vertical or horizontal line, or (only
//   for four) form a pinwheel round an empty hole. cover(points, k) tries every cut between
//   two distinct x (or y) coordinates and every split j + (k - j), recursing on both sides;
//   k = 1 is the bounding box. For k = 4 `pinwheel` also tries every empty hole x1 < x <= x2,
//   y1 < y <= y2 at point coordinates, in both turning directions; each arm is a quadrant, so
//   precomputed quadrant areas price a hole in O(1). O(n^4) for the pinwheels.
// original: corpus/submissions/5096459.cpp (k <= 3)
// Bounds: n: 1..=50; k: 1..=4; given.x, given.y: 0..=500
let n: 1..=50 = read()
let k: 1..=4 = read()
let given: [n](x: 0..=500, y: 0..=500) = read()

type Box = (xl: Int, xr: Int, yl: Int, yr: Int)

fn grow(b: ?Box, p: (Int, Int)) -> Box:
    let (x, y) = p
    if let (xl, xr, yl, yr) = b: (min(xl, x), max(xr, x), min(yl, y), max(yr, y)) else: (x, x, y, y)

fn size(b: ?Box) -> Int:
    if let (xl, xr, yl, yr) = b: (xr - xl) * (yr - yl) else: 0

fn area(ps: List<(Int, Int)>) -> Int:
    var b: ?Box = None
    for p in ps: b = Some(grow(b, p))
    size(b)

// area of the points on one side of x and one side of y, for every x in xs and y in ys
fn quadrant(
    ps: List<(Int, Int)>,
    xs: List<Int>,
    ys: List<Int>,
    right: Bool,
    up: Bool,
) -> List<List<Int>>:
    [[area([p for p in ps if (p.0 > x) == right and (p.1 > y) == up]) for y in ys] for x in xs]

// four arms round the empty hole x1 < x <= x2, y1 < y <= y2: top-left x <= x2, y > y2; top-right
// x > x2, y > y1; bottom-right x > x1, y <= y1; bottom-left x <= x1, y <= y2. Each arm is a
// quadrant, so tables of quadrant areas and of point counts make every hole O(1).
fn pinwheel(ps: List<(Int, Int)>) -> ?Int:
    let xs = [p.0 for p in ps].sorted().deduped()
    let ys = [p.1 for p in ps].sorted().deduped()
    let top_left = quadrant(ps, xs, ys, false, true)
    let top_right = quadrant(ps, xs, ys, true, true)
    let bottom_right = quadrant(ps, xs, ys, true, false)
    let bottom_left = quadrant(ps, xs, ys, false, false)
    let below = [[ps.count(p => p.0 <= x and p.1 <= y) for y in ys] for x in xs]
    var best: ?Int = None
    for i2 in 0..xs.len:
        for i1 in 0..i2:
            for j2 in 0..ys.len:
                for j1 in 0..j2:
                    if below[i2][j2] - below[i1][j2] - below[i2][j1] + below[i1][j1] == 0:
                        best min= top_left[i2][j2] +
                            top_right[i2][j1] +
                            bottom_right[i1][j1] +
                            bottom_left[i1][j2]
    best

// areas of the bounding boxes of s[0..i], for i in 0..=s.len
fn prefix_areas(s: List<(Int, Int)>) -> List<Int>:
    var b: ?Box = None
    var out = [0]
    for p in s:
        b = Some(grow(b, p))
        out.push(size(b))
    out

// least total area of at most `k` rectangles; a side of a cut left with one rectangle is its
// prefix or suffix bounding box, so only the sides with more recurse
fn cover(ps: List<(Int, Int)>, k: Int) -> Int:
    var best = area(ps)
    if k == 1: return best
    for turn in 0..2:
        let s = [if turn == 0: p else: (p.1, p.0) for p in ps].sorted()
        let pre = prefix_areas(s)
        let suf = prefix_areas(s.reversed()).reversed()
        for i in 1..s.len:
            if s[i - 1].0 < s[i].0:
                for j in 1..k:
                    let left = if j == 1: pre[i] else: cover(s[0..i].collect(), j)
                    let right = if j == k - 1: suf[i] else: cover(s[i..s.len].collect(), k - j)
                    best min= left + right
    if k == 4:
        if let w = pinwheel(ps): best min= w
        if let w = pinwheel([(-p.0, p.1) for p in ps]): best min= w  // the other turning direction
    best

print(cover([(p.x, p.y) for p in given], k))

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

// four arms round the hole x1 < x <= x2, y1 < y <= y2: the top arm takes x <= x2, the right
// one x > x2 down to y1, the bottom one x > x1, the left one the rest; None inside the hole
fn arm(p: (Int, Int), x1: Int, x2: Int, y1: Int, y2: Int) -> ?0..4:
    let (x, y) = p
    if y > y2: Some(if x <= x2: 0 else: 1)
    elif y > y1: (if x > x2: Some(1) elif x <= x1: Some(3) else: None)
    else: Some(if x > x1: 2 else: 3)

fn pinwheel(ps: List<(Int, Int)>) -> ?Int:
    let xs = [p.0 for p in ps].sorted().deduped()
    let ys = [p.1 for p in ps].sorted().deduped()
    var best: ?Int = None
    for x1 in xs:
        for x2 in xs:
            for y1 in ys:
                for y2 in ys:
                    if x1 < x2 and y1 < y2:
                        var box: [0..4]?Box = None
                        var hole = false
                        for p in ps:
                            if let a = arm(p, x1, x2, y1, y2): box[a] = Some(grow(box[a], p))
                            else:
                                hole = true
                                break
                        if not hole: best min= sum(size(box[a]) for a in 0..4)
    best

// least total area of at most `k` rectangles
fn cover(ps: List<(Int, Int)>, k: Int) -> Int:
    var best = area(ps)
    if k == 1: return best
    for turn in 0..2:
        let s = [if turn == 0: p else: (p.1, p.0) for p in ps].sorted()
        for i in 1..s.len:
            if s[i - 1].0 < s[i].0:
                let (a, b) = (s[0..i].collect(), s[i..s.len].collect())
                for j in 1..k: best min= cover(a, j) + cover(b, k - j)
    if k == 4:
        if let w = pinwheel(ps): best min= w
        if let w = pinwheel([(-p.0, p.1) for p in ps]): best min= w  // the other turning direction
    best

print(cover([(p.x, p.y) for p in given], k))

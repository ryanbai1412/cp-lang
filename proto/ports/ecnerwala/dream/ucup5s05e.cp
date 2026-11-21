// Universal Cup 5 Stage 5 (Chengdu) E: for each query point, name three of
// the given points whose triangle strictly contains it (or the four hull
// corners when only that quadrilateral does), else -1. Only a few hull
// vertices around the query's x, plus one point off both diagonals of a
// four-corner hull, can matter, so every triple of those is tried.
use std.geometry.{convex_hull_idx, turn}

fn in_tri(a: Pt, b: Pt, d: Pt, c: Pt) -> Bool:
    let s = turn(a, b, d)
    s != 0 and s == turn(a, b, c) and s == turn(a, c, d) and s == turn(c, b, d)

@!multitest
let (n, q): (3..=2e5, 1..=2e5) = read()
domain P = 1..=n
let pts: [P]Pt = read()

let hull = convex_hull_idx(pts, collinear: true)
let rm = (0..hull.len).max(by: i => pts[hull[i]])!
let lower = hull[..=rm].to_list()
let upper = [hull[0]] + hull[rm..].rev.to_list()

fn off_both_diags(p: P) -> Bool:
    let (h0, h1, h2, h3) = (pts[hull[0]], pts[hull[1]], pts[hull[2]], pts[hull[3]])
    turn(h0, h2, pts[p]) != 0 and turn(h1, h3, pts[p]) != 0

let off_diagonal: ?P = if hull.len == 4: first(p for p in P if off_both_diags(p)) else: none
let quad_edges = if hull.len == 4:
    [(hull[0], hull[1]), (hull[1], hull[2]), (hull[2], hull[3]), (hull[3], hull[0])]
else:
    []

// Up to five consecutive chain vertices around where `c` falls in x order.
fn near(ch: List<P>, c: Pt) -> List<P>:
    let at = bsearch_first(0..ch.len, i => pts[ch[i]] >= c)!
    let hi = min(ch.len, at + 4)
    let lo = max(0, min(at - 1, hi - 5))
    ch[lo..hi].to_list()

fn answer(c: Pt) -> List<P>:
    if c.x <= pts[lower.first!].x or c.x >= pts[lower.last!].x: return []
    var cands = near(lower, c) + near(upper, c)
    if let s = off_diagonal: cands.push(s)
    if let tri = cands.combinations(3).first(t => in_tri(pts[t[0]], pts[t[1]], pts[t[2]], c)):
        return tri
    if hull.len == 4 and off_diagonal == none and quad_edges.all(
        e => turn(pts[e.0], pts[e.1], c) > 0
    ):
        return hull
    []

for _ in 0..q:
    let p: Pt = read()
    let ids = answer(p)
    if ids.is_empty: print(-1)
    else:
        print(ids.len)
        print(ids)

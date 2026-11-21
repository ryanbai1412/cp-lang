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
var rm = 0
for i in 1..hull.len:
    if pts[hull[i]] > pts[hull[rm]]: rm = i
let lower = hull[..=rm].to_list()
let upper = [hull[0]] + hull[rm..].rev.to_list()
let off_diagonal: ?P = if hull.len == 4:
    first(
        p for p in P if turn(pts[hull[0]], pts[hull[2]], pts[p]) != 0 and turn(
            pts[hull[1]],
            pts[hull[3]],
            pts[p],
        ) != 0
    )
else: none

// Up to five consecutive chain vertices around where `c` falls in x order.
fn near(ch: List<P>, c: Pt) -> List<P>:
    let at = bsearch_first(0..ch.len, i => pts[ch[i]] >= c)!
    var (lo, hi) = (at - 1, at + 1)
    while hi - lo < 5 and (hi < ch.len or lo > 0):
        if hi < ch.len: hi += 1 else: lo -= 1
    ch[lo..hi].to_list()

fn answer(c: Pt) -> List<P>:
    if c.x <= pts[lower.first!].x or c.x >= pts[lower.last!].x: return []
    var cands = near(lower, c) + near(upper, c)
    if let s = off_diagonal: cands.push(s)
    for i in 0..cands.len:
        for j in i + 1..cands.len:
            for k in j + 1..cands.len:
                if in_tri(pts[cands[i]], pts[cands[j]], pts[cands[k]], c):
                    return [cands[i], cands[j], cands[k]]
    if hull.len == 4 and off_diagonal == none and all(
        turn(pts[hull[i]], pts[hull[(i + 1) % 4]], c) > 0 for i in 0..4
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

// Universal Cup 5 Stage 5 (Chengdu) E: for each query point, name three of
// the given points whose triangle strictly contains it (or the four hull
// corners when only that quadrilateral does), else -1. Only a few hull
// vertices around the query's x, plus one point off both diagonals of a
// four-corner hull, can matter, so every triple of those is tried.
use std.geometry.{turn}

type Site = (p: Pt, id: Int)

@! multitest
let (n, q): (3..=2e5, 1..=2e5) = read()
let pts: [0..n]Pt = read()
let sites = sorted([(p: pts[i], id: i + 1) for i in 0..n], by: s => s.p)

// The lower (side = 1) or upper (side = -1) hull, left to right, collinear points kept.
fn chain(side: Int) -> List<Site>:
    var c: [0..n]Site = sites[0]
    var len = 0
    for s in sites:
        while len >= 2 and turn(c[len - 2].p, c[len - 1].p, s.p) == -side: len -= 1
        c[len] = s
        len += 1
    [c[i] for i in 0..len]

let (lower, upper) = (chain(1), chain(-1))
let hull = lower[..lower.len - 1] + upper[1..].rev
let off_diagonal: ?Site = if hull.len == 4:
    first(
        s for s in sites if turn(hull[0].p, hull[2].p, s.p) != 0 and turn(
            hull[1].p,
            hull[3].p,
            s.p,
        ) != 0
    )
else: none

// Up to five consecutive chain vertices around where `c` falls in x order.
fn near(ch: List<Site>, c: Pt) -> List<Site>:
    let at = bsearch_first(0..ch.len, i => ch[i].p >= c)!
    var (lo, hi) = (at - 1, at + 1)
    while hi - lo < 5 and (hi < ch.len or lo > 0):
        if hi < ch.len: hi += 1 else: lo -= 1
    ch[lo..hi]

fn answer(c: Pt) -> List<Int>:
    if c.x <= lower.first!.p.x or c.x >= lower.last!.p.x: return []
    var cands = near(lower, c) + near(upper, c)
    if let s = off_diagonal: cands.push(s)
    for i in 0..cands.len:
        for j in i + 1..cands.len:
            for k in j + 1..cands.len:
                let (a, b, d) = (cands[i].p, cands[j].p, cands[k].p)
                let s = turn(a, b, d)
                if s != 0 and s == turn(a, b, c) and s == turn(a, c, d) and s == turn(c, b, d):
                    return [cands[i].id, cands[j].id, cands[k].id]
    if hull.len == 4 and off_diagonal == none and all(
        turn(hull[i].p, hull[(i + 1) % 4].p, c) > 0 for i in 0..4
    ):
        return [h.id for h in hull]
    []

for _ in 0..q:
    let p: Pt = read()
    let ids = answer(p)
    if ids.is_empty: print(-1)
    else:
        print(ids.len)
        print(ids)

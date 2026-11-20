// Universal Cup 5 Stage 5 (Chengdu) E: for each query point, name three of
// the given points whose triangle strictly contains it (or the four hull
// corners when only that quadrilateral does), else -1. Only a few hull
// vertices around the query's x, plus one point off both diagonals of a
// four-corner hull, can matter, so every triple of those is tried.
use std.geometry.{Pt, turn}
use std.seq.{bsearch_first}
type Site = (p: Pt, id: Int)
@!multitest
let (n, q): (3..=2e5, 1..=2e5) = read()
let input: [0..n](x: Int, y: Int) = read()
let sites = sorted([(p: Pt(input[i].x, input[i].y), id: i + 1) for i in 0..n], by: s => (s.p.x, s.p.y))

// The lower (side = 1) or upper (side = -1) hull, left to right, collinear points kept.
fn chain(side: Int) -> List<Site>:
    var c = List<Site>()
    for s in sites:
        while c.len >= 2 and turn(c[c.len - 2].p, c[c.len - 1].p, s.p) == -side: c.pop()
        c.push(s)
    c

let (lower, upper) = (chain(1), chain(-1))
let hull = lower[0..lower.len - 1] + [upper[i] for i in (1..upper.len).rev]
var off_diagonal: ?Site = none
if hull.len == 4:
    for s in sites if turn(hull[0].p, hull[2].p, s.p) != 0 and turn(hull[1].p, hull[3].p, s.p) != 0:
        off_diagonal = s
        break

// Up to five consecutive chain vertices around where `c` falls in x order.
fn near(chain: List<Site>, c: Pt) -> List<Site>:
    let at = bsearch_first(0..chain.len, i => (chain[i].p.x, chain[i].p.y) >= (c.x, c.y))!
    var (lo, hi) = (at - 1, at + 1)
    while hi - lo < 5 and (hi < chain.len or lo > 0):
        if hi < chain.len: hi += 1 else: lo -= 1
    chain[lo..hi]

fn answer(c: Pt) -> List<Int>:
    if c.x <= lower.first!.p.x or c.x >= lower.last!.p.x: return []
    var cands = near(lower, c) + near(upper, c)
    if off_diagonal != none: cands.push(off_diagonal!)
    for i in 0..cands.len:
        for j in i + 1..cands.len:
            for k in j + 1..cands.len:
                let (a, b, d) = (cands[i].p, cands[j].p, cands[k].p)
                let s = turn(a, b, d)
                if s != 0 and s == turn(a, b, c) and s == turn(a, c, d) and s == turn(c, b, d):
                    return [cands[i].id, cands[j].id, cands[k].id]
    if hull.len == 4 and off_diagonal == none and all(turn(hull[i].p, hull[(i + 1) % 4].p, c) > 0 for i in 0..4):
        return [h.id for h in hull]
    []

for _ in 0..q:
    let (x, y): (Int, Int) = read()
    let ids = answer(Pt(x, y))
    if ids.len == 0: print(-1)
    else:
        print(ids.len)
        print(ids)

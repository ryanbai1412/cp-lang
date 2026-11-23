// Source: Mock CCC '22 2 S4 (Kaguya Wants to Build a Great Wall), https://dmoj.ca/problem/nccc10s4
// Problem: 3 <= n <= 12 distinct lines, each through two lattice points with |coordinate| <= 2000.
//   Count the distinct simple n-sided polygons with exactly one side (of positive length) on
//   each line.
// Solution: corners are crossings of two lines, kept exactly as reduced (x, y, w) for
//   (x / w, y / w) and compressed into a domain sorted by x, then y: the order along every line.
//   Backtracking walk: from a corner take an unused line and slide either way to a later
//   crossing, the next corner; every crossing passed is spent, so the polygon cannot touch
//   itself, and it closes when the last line runs into the start. The first side lies on the
//   first line and runs forwards, so each polygon counts once. Exponential in n, fine for 12.
// original: corpus/submissions/4225209.cpp
use std.geometry.{cross}
type Coord = -2000..=2000
struct Line(at: Pt, d: Pt)
struct Corner(x: Int, y: Int, w: Int)  // (x / w, y / w) with w > 0 and gcd 1: one triple per point

impl Ord for Corner:
    fn lt(p, q) = (p.x * q.w, p.y * q.w) < (q.x * p.w, q.y * p.w)

fn crossing(p: Line, q: Line) -> ?Corner:
    let w = cross(p.d, q.d)
    if w == 0: return None  // parallel, or the same line
    let h = p.at * w + p.d * cross(q.at - p.at, q.d)
    let g = gcd(gcd(abs(h.x), abs(h.y)), abs(w)) * Int(w.sign)
    Corner(h.x / g, h.y / g, w / g)

let n = read<3..=12>()
domain W = [n]
let ends = read<[W](a: (Coord, Coord), b: (Coord, Coord))>()
let line = [W] w => Line(Pt(ends[w].a.0, ends[w].a.1), Pt(ends[w].b.0 - ends[w].a.0, ends[w].b.1 - ends[w].a.1))
fn crossings(u: W) -> yields Corner = W.filter_map(v => crossing(line[u], line[v]))
let (domain C, corner) = compress([p for u in W for p in crossings(u)])
let along = [W] u => crossings(u).map(p => corner.index_of(p)).sorted().deduped()

var used: [W]Bool = false
var spent: [C]Bool = false
var polygons = 0
fn walk(from: C, start: C, sides: Int):
    for w in W:
        if used[w] or not along[w].contains(from): continue
        used[w] = true
        for ray in [along[w].filter(c => c > from).collect(), along[w].filter(c => c < from).rev.collect()]:
            var side: List<C> = []
            for c in ray:
                if spent[c]:
                    if c == start and sides + 1 == n: polygons += 1
                    break
                spent[c] = true
                side.push(c)
                if sides + 1 < n: walk(c, start, sides + 1)
            for c in side: spent[c] = false
        used[w] = false

let first = W.first!
used[first] = true
for (s, t) in along[first].pairs():
    for c in along[first]: spent[c] = s <= c <= t
    walk(t, s, 1)
print(polygons)

// Source: Mock CCC '22 2 S4 (Kaguya Wants to Build a Great Wall), https://dmoj.ca/problem/nccc10s4
// Problem: n <= 12 distinct lines, each given by two lattice points with |coordinate| <= 2000.
//   Count the distinct simple n-sided polygons with exactly one side (of positive length) on
//   each line.
// Solution: corners are crossings of two lines, kept as reduced whole-number triples (x, y, w)
//   for (x / w, y / w): equal points have equal triples and order along a line is one product
//   comparison, so nothing is divided. Backtracking walk: from a corner take an unused line and
//   slide either way to a later crossing, which becomes the next corner; every crossing passed is
//   spent, so the polygon cannot touch itself. The first side lies on line 0 and runs forwards,
//   counting each polygon once. Exponential in n, fine for n <= 12.
// original: corpus/submissions/4225209.cpp
// Bounds: n: 3..=12

struct Line(a: Int, b: Int, c: Int)  // a x + b y + c = 0
struct Hom(x: Int, y: Int, w: Int)   // the point (x / w, y / w), w > 0, reduced

fn crossing(p: Line, q: Line) -> ?Hom:
    let w = p.a * q.b - q.a * p.b
    if w == 0: return None  // parallel, or the same line
    let (x, y) = (p.b * q.c - q.b * p.c, p.c * q.a - q.c * p.a)
    let g = gcd(gcd(abs(x), abs(y)), abs(w)) * (if w < 0: -1 else: 1)
    Hom(x / g, y / g, w / g)

impl Ord for Hom:
    fn lt(p, q) = if p.x * q.w != q.x * p.w: p.x * q.w < q.x * p.w else: p.y * q.w < q.y * p.w

let n: 3..=12 = read()
domain L = 0..n
type Coord = -2000..=2000
let ends: [L](x1: Coord, y1: Coord, x2: Coord, y2: Coord) = read()
fn line_through(i: L) -> Line:
    let e = ends[i]
    Line(e.y1 - e.y2, e.x2 - e.x1, e.x1 * e.y2 - e.x2 * e.y1)
let lines: [L]Line = [L] i => line_through(i)

// every crossing is one corner; each line keeps its corners in order
fn corners_on(i: L) -> List<Hom>:
    var found = L.filter_map(j => crossing(lines[i], lines[j])).collect()
    found.sort()
    found.dedup()
    found
let hits: [L]List<Hom> = [L] i => corners_on(i)
var id: Map<Hom, Int> = Map()
for i in L:
    for p in hits[i]:
        if not id.contains(p): id[p] = id.len
domain Corner = 0..id.len
let along: [L]List<Corner> = [L] i => hits[i].map(p => Corner(id[p])).collect()
var place: Map<(Corner, L), Int> = Map()  // where a corner sits in along[line]
for i in L:
    for (k, c) in along[i].items: place[(c, i)] = k

var used: [L]Bool = false
var spent: [Corner]Bool = false
var (sides, polygons) = (1, 0)

// from `corner`, take an unused line and slide either way to a later corner; every corner
// passed is spent
fn walk(corner: Corner, start: Corner):
    if sides == n: return
    for i in L:
        if used[i]: continue
        let here = place.get((corner, i)) else: continue
        used[i] = true
        sides += 1
        for dir in [1, -1]:
            var k = here + dir
            while let next = along[i].get(k):
                if sides == n and next == start: polygons += 1
                if spent[next]: break
                spent[next] = true
                walk(next, start)
                k += dir
            var j = here + dir
            while j != k:
                spent[along[i][j]] = false
                j += dir
        sides -= 1
        used[i] = false

// the first side lies on line 0 and runs forwards
used[L.first!] = true
let base = along[L.first!]
for (k, start) in base.items:
    spent.fill(false)
    spent[start] = true
    for c in base.drop(k + 1):
        spent[c] = true
        walk(c, start)
print(polygons)

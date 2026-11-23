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

struct Line(a: Int, b: Int, c: Int)  // a x + b y + c = 0
struct Hom(x: Int, y: Int, w: Int)   // the point (x / w, y / w), w > 0, reduced

fn through(x1: Int, y1: Int, x2: Int, y2: Int) -> Line = Line(y1 - y2, x2 - x1, x1 * y2 - x2 * y1)

fn crossing(p: Line, q: Line) -> ?Hom:
    let w = p.a * q.b - q.a * p.b
    if w == 0: return None  // parallel, or the same line
    let (x, y) = (p.b * q.c - q.b * p.c, p.c * q.a - q.c * p.a)
    let g = gcd(gcd(abs(x), abs(y)), abs(w)) * (if w < 0: -1 else: 1)
    Hom(x / g, y / g, w / g)

impl Ord for Hom:
    fn lt(p, q) = if p.x * q.w != q.x * p.w: p.x * q.w < q.x * p.w else: p.y * q.w < q.y * p.w

let n: Int = read()
var lines: List<Line> = []
for _ in 0..n:
    let (x1, y1, x2, y2): (Int, Int, Int, Int) = read()
    lines.push(through(x1, y1, x2, y2))

var id_of: Map<(Int, Int, Int), Int> = Map()
var along: List<List<Int>> = []          // the crossings on each line, in order
var place: Map<(Int, Int), Int> = Map()  // (crossing, line) -> where it is in along[line]
for i in 0..n:
    var found: List<Hom> = []
    for other in lines:
        if let p = crossing(lines[i], other): found.push(p)
    found.sort()
    var ids: List<Int> = []
    for p in found:
        let key = (p.x, p.y, p.w)
        if not id_of.contains(key): id_of[key] = id_of.len
        let id = id_of[key]
        if ids.is_empty or ids.last != id:
            place[(id, i)] = ids.len
            ids.push(id)
    along.push(ids)

var used: List<Bool> = List.filled(n, false)
var spent: List<Bool> = List.filled(id_of.len, false)
var (sides, start, polygons) = (1, 0, 0)

fn walk(corner: Int):
    if sides == n: return
    for i in 0..n:
        if used[i]: continue
        let here = place.get((corner, i)) else: continue
        used[i] = true
        sides += 1
        for step in [1, -1]:
            var passed: List<Int> = []
            var j = here + step
            while 0 <= j and j < along[i].len:
                let next = along[i][j]
                if sides == n and next == start: polygons += 1
                if spent[next]: break
                spent[next] = true
                passed.push(next)
                walk(next)
                j += step
            for v in passed: spent[v] = false
        sides -= 1
        used[i] = false

if n > 0:
    used[0] = true
    for i in 0..along[0].len:
        start = along[0][i]
        for v in 0..spent.len: spent[v] = false
        spent[start] = true
        for j in i + 1..along[0].len:
            spent[along[0][j]] = true
            walk(along[0][j])
print(polygons)

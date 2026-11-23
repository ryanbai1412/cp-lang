// nccc10s4: n lines, each through two lattice points. Count the simple
// polygons with n sides that have exactly one side on each line.
//
// Brute force: try every order of the lines that starts with line 0. Side k
// lies on line order[k] and runs between its crossings with the lines before
// and after it. The order is a simple polygon when every corner exists, no
// side is a single point, and no two non-adjacent sides touch. Each polygon
// is found twice, once per direction.

struct Line(a: Int, b: Int, c: Int)         // a x + b y + c = 0
struct Hom(x: Int, y: Int, w: Int)          // the point (x / w, y / w), w > 0, reduced

fn crossing(p: Line, q: Line) -> ?Hom:
    let w = p.a * q.b - q.a * p.b
    if w == 0: return none
    let (x, y) = (p.b * q.c - q.b * p.c, p.c * q.a - q.c * p.a)
    let g = gcd(gcd(abs(x), abs(y)), abs(w)) * (if w < 0: -1 else: 1)
    Hom(x / g, y / g, w / g)

impl Ord for Hom:
    fn lt(p, q) = if p.x * q.w != q.x * p.w: p.x * q.w < q.x * p.w else: p.y * q.w < q.y * p.w

fn same(p: Hom, q: Hom) -> Bool = p.x == q.x and p.y == q.y and p.w == q.w

// For points on one line: q lies on the side from s to t.
fn between(q: Hom, s: Hom, t: Hom) -> Bool = not (q < s and q < t) and not (s < q and t < q)

fn coincide(p: Line, q: Line) -> Bool = p.a * q.b == q.a * p.b and p.a * q.c == q.a * p.c and p.b * q.c == q.b * p.c

let n: Int = read()
var lines: List<Line> = []
for _ in 0..n:
    let (x1, y1, x2, y2): (Int, Int, Int, Int) = read()
    lines.push(Line(y1 - y2, x2 - x1, x1 * y2 - x2 * y1))

fn simple(order: List<Int>) -> Bool:
    var corner: List<Hom> = []
    for k in 0..n:
        let p = crossing(lines[order[k]], lines[order[(k + 1) % n]]) else: return false
        corner.push(p)
    fn from(k: Int) -> Hom = corner[(k + n - 1) % n]
    for k in 0..n:
        if same(from(k), corner[k]): return false
    for j in 0..n:
        for k in j + 2..n:
            if j == 0 and k == n - 1: continue
            let (p, q) = (lines[order[j]], lines[order[k]])
            if let c = crossing(p, q):
                if between(c, from(j), corner[j]) and between(c, from(k), corner[k]): return false
            if coincide(p, q):
                if between(from(j), from(k), corner[k]) or between(corner[j], from(k), corner[k]) or between(from(k), from(j), corner[j]): return false
    true

var found = 0
for order in [i for i in 0..n].permutations():
    if order[0] == 0 and simple(order): found += 1
print(found / 2)

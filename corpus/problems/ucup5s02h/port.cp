// Universal Cup 5, stage 2 H: split charges on a line into two groups (some
// charges are pinned to A or B) so that the weakest pull q * q' / d^2 between
// two charges of one group is as strong as possible; print it as a fraction.
//
// As for any two-clustering that wants its weakest same-group pair strong, it
// is enough to two-colour the spanning tree of weakest pulls: pinned charges
// hang off a reference node `pin` on A's or B's side. Boruvka builds that tree.
// Each round, a charge finds its weakest pull to another component among the
// charges left of it (then, mirrored, right of it): a segment tree over
// components keeps a `Hull` per node, and climbing from the charge's own leaf
// queries every sibling, i.e. every other component.
//
// `Hull` holds charges added left to right and answers "which of them pulls
// weakest on x" for increasing x. A charge no weaker than the newest and
// farther away never wins, so charges strictly decrease, and the weakest one
// only moves right as x does.
use std.dsu.{ParityDsu}

struct Pt(x: Int, q: Int, id: Int)

fn no_stronger(b: Pt, a: Pt, x: Int) -> Bool:
    let (da, db) = (a.x - x, b.x - x)
    da * da * b.q <= db * db * a.q

fn never_weakest(a: Pt, b: Pt, c: Pt) -> Bool:
    let (v1, v2, v3) = (c.x - b.x, c.x - a.x, b.x - a.x)
    let (a1, a2, a3) = (v1 * v1 * a.q, v2 * v2 * b.q, v3 * v3 * c.q)
    if a2 < a1 + a3: return false
    let d = I128(a2) - I128(a1) - I128(a3)
    d * d >= I128(4) * I128(a1) * I128(a3)

struct Hull:
    pts: List<Pt>
    idx: Int

    fn new() -> Hull = Hull([], 0)

    fn advance(mut self, x: Int):
        while idx + 1 < pts.len and no_stronger(pts[idx + 1], pts[idx], x): idx += 1

    fn add(mut self, p: Pt):
        if pts.is_empty or p.q < pts.last!.q:
            while idx + 1 < pts.len and never_weakest(pts[pts.len - 2], pts.last!, p):
                pts.pop()
            pts.push(p)
        self.advance(p.x)

    fn query(mut self, x: Int) -> ?Pt:
        self.advance(x)
        if idx == pts.len: none else: some(pts[idx])

fn weaker(a: ?Pt, b: ?Pt, x: Int) -> ?Pt:
    let pa = a else: return b
    let pb = b else: return a
    if no_stronger(pb, pa, x): b else: a

// d^2 / (q * q'), compared by cross-multiplying; larger is weaker.
fn gap(a: Pt, b: Pt) -> (num: Int, den: Int) = ((a.x - b.x) * (a.x - b.x), a.q * b.q)
fn weaker_gap(
    a: (num: Int, den: Int),
    b: (num: Int, den: Int),
) -> Bool = a.num * b.den >= a.den * b.num

@!multitest
let n: 2..=150_000 = read()
var charges = List<(x: Int, q: Int, pin: Char)>()
for _ in 0..n:
    let (q, x, c): (Int, Int, Char) = read()
    charges.push((x, q, c))
charges.sort()

domain C = 0..n
let pts = [C] i => Pt(charges[i.ord].x, charges[i.ord].q, i.ord)

domain Node = 0..=n
let pin = Node.at(n)
fn node_of(i: C) -> Node = Node.at(i.ord)!
var colour = ParityDsu<Node>()
let pinned = [i for i in C if charges[i.ord].pin == 'A' or charges[i.ord].pin == 'B']
for i in pinned: let _ = colour.unite(node_of(i), pin!, differ: charges[i.ord].pin == 'A')
if pinned.is_empty: let _ = colour.unite(node_of(C.first!), pin!, differ: false)

fn link(i: C, j: C):
    if not colour.same(node_of(i), node_of(j)): let _ = colour.unite(node_of(i), node_of(j))

while not C.all(i => colour.same(node_of(i), node_of(C.first!))):
    let roots = [u for u in Node if colour.find(u).root == u]
    var comp: [Node]Int = 0
    for (k, r) in roots.items: comp[r] = k
    let v = roots.len
    var best: [0..v]?(gap: (num: Int, den: Int), i: C, j: C) = none
    fn sweep(order: List<C>, sign: Int):
        var hulls = [Hull.new() for _ in 0..2 * v]
        fn ancestors(cc: Int) -> List<Int>:
            var out: List<Int> = []
            var node = cc + v
            while node > 1:
                out.push(node)
                node >>= 1
            out
        for i in order:
            let x = sign * pts[i].x
            let cc = comp[colour.find(node_of(i)).root]
            let path = ancestors(cc)
            var hit: ?Pt = none
            for node in path: hit = weaker(hit, hulls[node ^ 1].query(x), x)
            if let h = hit:
                let j = C.at(h.id)
                let cand = gap(pts[j!], pts[i])
                let wins = if let cur = best[cc]: weaker_gap(cand, cur.gap) else: true
                if wins: best[cc] = some((cand, i, j!))
            for node in path: hulls[node].add(Pt(x, pts[i].q, i.ord))
    sweep([i for i in C], 1)
    sweep([i for i in C.rev], -1)
    for e in best if let edge = e: link(edge.i, edge.j)

var weakest: ?(num: Int, den: Int) = none
for side in [false, true]:
    var hull = Hull.new()
    for i in C if (colour.diff(node_of(i), pin!) == some(false)) == side:
        let p = pts[i]
        if let prev = hull.query(p.x):
            let g = gap(prev, p)
            if weakest == none or not weaker_gap(weakest!, g): weakest = some(g)
        hull.add(p)

// Fallback only triggers when n = 2 puts one charge on each side (no same-side pair).
let w = weakest ?? (num: 1, den: 226_000_000)
let g = gcd(w.den, w.num)
print("{w.den / g}/{w.num / g}")

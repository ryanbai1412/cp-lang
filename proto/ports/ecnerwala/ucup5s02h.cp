fn cmp(ax: Int, aq: Int, bx: Int, bq: Int, x: Int) -> Bool:
    let (da, db) = (ax - x, bx - x)
    da * da * bq <= db * db * aq

fn cmp3(ax: Int, aq: Int, bx: Int, bq: Int, cx: Int, cq: Int) -> Bool:
    let (v1, v2, v3) = (cx - bx, cx - ax, bx - ax)
    let (a1, a2, a3) = (v1 * v1 * aq, v2 * v2 * bq, v3 * v3 * cq)
    if a2 < a1 + a3: return false
    let d = I128(a2) - I128(a1) - I128(a3)
    d * d >= I128(4) * I128(a1) * I128(a3)

struct Hull:
    pts: List<(Int, Int, Int)>
    idx: Int

    fn add(mut self, x: Int, q: Int, id: Int):
        if pts.is_empty or q < pts.last!.1:
            while idx + 1 < pts.len:
                let p0 = pts[pts.len - 2]
                let p1 = pts.last!
                if not cmp3(p0.0, p0.1, p1.0, p1.1, x, q): break
                let _ = pts.pop()
            pts.push((x, q, id))
        while idx + 1 < pts.len and cmp(pts[idx].0, pts[idx].1, pts[idx + 1].0, pts[idx + 1].1, x):
            idx += 1

    fn query(mut self, cx: Int) -> ?(Int, Int, Int):
        while idx + 1 < pts.len and cmp(pts[idx].0, pts[idx].1, pts[idx + 1].0, pts[idx + 1].1, cx):
            idx += 1
        if idx == pts.len: none else: some(pts[idx])

fn pick_best(a: ?(Int, Int, Int), b: ?(Int, Int, Int), x: Int) -> ?(Int, Int, Int):
    let pa = a else: return b
    let pb = b else: return a
    if cmp(pa.0, pa.1, pb.0, pb.1, x): b else: a

@!multitest
let n: Int = read()
var charges = List<(Int, Int, Char)>()
for _ in 0..n:
    let (q, x, c): (Int, Int, Char) = read()
    charges.push((x, q, c))
charges.sort()

let xs = [c.0 for c in charges]
let qs = [c.1 for c in charges]
domain Node = 0..=n
domain Bip = 0..2 * n + 2
var uf = Dsu<Node>()
var bip = Dsu<Bip>()
var has_letter = false
for i in 0..n:
    let c = charges[i].2
    if c == 'A':
        let _ = bip.merge(Bip.at(2 * i), Bip.at(2 * n + 1))
        let _ = bip.merge(Bip.at(2 * i + 1), Bip.at(2 * n))
        let _ = uf.merge(Node.at(n), Node.at(i))
        has_letter = true
    elif c == 'B':
        let _ = bip.merge(Bip.at(2 * i), Bip.at(2 * n))
        let _ = bip.merge(Bip.at(2 * i + 1), Bip.at(2 * n + 1))
        let _ = uf.merge(Node.at(n), Node.at(i))
        has_letter = true
if not has_letter:
    let _ = bip.merge(Bip.at(0), Bip.at(2 * n))
    let _ = bip.merge(Bip.at(1), Bip.at(2 * n + 1))
    let _ = uf.merge(Node.at(0), Node.at(n))

while not all(uf.same(Node.at(i), Node.at(0)) for i in 0..n):
    var cc_id: [Node]Int = -1
    var v = 0
    for u in Node:
        if uf.find(u) == u:
            cc_id[u] = v
            v += 1
    var best_per: [0..v]?(Int, Int, Int, Int) = none
    for dir in 0..2:
        var hulls = [Hull(List<(Int, Int, Int)>(), 0) for _ in 0..2 * v]
        for it in 0..n:
            let id = if dir == 0: it else: n - 1 - it
            let x = if dir == 0: xs[id] else: -xs[id]
            let cc = cc_id[uf.find(Node.at(id))]
            var best: ?(Int, Int, Int) = none
            var p = cc + v
            while p > 1:
                best = pick_best(best, hulls[p ^ 1].query(x), x)
                p >>= 1
            if let hit = best:
                let j = hit.2
                let (sx, sy) = ((xs[j] - xs[id]) * (xs[j] - xs[id]), qs[j] * qs[id])
                let take = if let cur = best_per[cc]: sx * cur.1 >= sy * cur.0 else: true
                if take: best_per[cc] = some((sx, sy, id, j))
            var q = cc + v
            while q > 1:
                hulls[q].add(x, qs[id], id)
                q >>= 1
    for opt_e in best_per:
        if let edge = opt_e:
            let (i, j) = (edge.2, edge.3)
            if uf.merge(Node.at(i), Node.at(j)) != none:
                let _ = bip.merge(Bip.at(2 * i), Bip.at(2 * j + 1))
                let _ = bip.merge(Bip.at(2 * i + 1), Bip.at(2 * j))

var side0 = List<(Int, Int)>()
var side1 = List<(Int, Int)>()
for i in 0..n:
    if bip.same(Bip.at(2 * i), Bip.at(2 * n)): side1.push((xs[i], qs[i]))
    else: side0.push((xs[i], qs[i]))

var (best_x, best_y) = (1, 226_000_000)
for side in [side0, side1]:
    var sh = Hull(List<(Int, Int, Int)>(), 0)
    for (id, pt) in side.items:
        let (x, q) = pt
        if let prv = sh.query(x):
            let (nx, ny) = ((x - prv.0) * (x - prv.0), q * prv.1)
            if nx * best_y > ny * best_x: (best_x, best_y) = (nx, ny)
        sh.add(x, q, id)

let g = gcd(best_y, best_x)
print("{best_y / g}/{best_x / g}")

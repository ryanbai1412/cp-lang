// expect: ok, output "ok"
// input: "1 300\n"
// model: Functional
// model: TwoSat
// model: Lit
// model: de_bruijn
// Functional graphs against walking `next` step by step, 2-SAT against trying all 16
// assignments of 4 variables, and de Bruijn sequences against the Lyndon words found by
// comparing every rotation. Reads `seed n`.
use std.graph.{Functional, Lit, TwoSat, de_bruijn}
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain V = 0..7
domain X = 0..4
fn walk(nx: List<Int>, v: Int, k: Int) -> Int:
    var at = v
    for _ in 0..k: at = nx[at]
    at
fn on_cycle(nx: List<Int>, v: Int) -> Bool = (1..8).any(l => walk(nx, v, l) == v)
var rng = Rng(seed0)
var bad = ""
for k in 0..n:
    let nx = [rng.int(0, 6) for _ in 0..7]
    let f = Functional([V] v => V(nx[v.ord]))
    for v in 0..7:
        var d = 0
        while not on_cycle(nx, walk(nx, v, d)): d += 1
        let r = walk(nx, v, d)
        var len = 1
        while walk(nx, r, len) != r: len += 1
        let steps = rng.int(0, 40)
        let to = rng.int(0, 6)
        var dist: ?Int = None
        for s in 0..15:
            if dist == None and walk(nx, v, s) == to: dist = Some(s)
        let cut = rng.int(0, 6)
        var jw: ?V = None
        for s in 0..15:
            if jw == None and walk(nx, v, s) >= cut and on_cycle(nx, walk(nx, v, s)): jw = Some(V(walk(nx, v, s)))
        let x = V(v)
        if f.on_cycle(x) != on_cycle(nx, v) or f.depth(x) != d or f.root_of(x) != V(r) or f.cycle_len(x) != len: bad = "op {k}: Functional {v}"
        elif f.after(x, steps) != V(walk(nx, v, steps)): bad = "op {k}: after {v} {steps}"
        elif f.dist(x, V(to)) != dist: bad = "op {k}: dist {v} {to}"
        elif f.jump_while(x, u => f.on_cycle(u)) != Some(V(r)): bad = "op {k}: jump_while {v}"
    var cyc = List<List<V>>()
    for v in 0..7:
        if on_cycle(nx, v) and (0..7).all(u => not (on_cycle(nx, u) and u < v and (1..8).any(l => walk(nx, u, l) == v))):
            var c = [V(v)]
            var at = nx[v]
            while at != v:
                c.push(V(at))
                at = nx[at]
            cyc.push(c)
    if f.cycles() != cyc: bad = "op {k}: cycles"
    // 2-SAT: clauses kept as literal pairs, one of which must hold
    var s = TwoSat<X>()
    var cl = List<((Int, Bool), (Int, Bool))>()
    for _ in 0..rng.int(0, 7):
        let op = rng.int(0, 5)
        let a = (rng.int(0, 3), rng.int(0, 1) == 1)
        let b = (rng.int(0, 3), rng.int(0, 1) == 1)
        let la = Lit(X(a.0), a.1)
        let lb = Lit(X(b.0), b.1)
        let na = (a.0, not a.1)
        let nb = (b.0, not b.1)
        if la.neg != Lit(X(a.0), not a.1) or la.neg.neg != la: bad = "op {k}: Lit.neg"
        if op == 0:
            s.add_or(la, lb)
            cl.push((a, b))
        elif op == 1:
            s.must(la)
            cl.push((a, a))
        elif op == 2:
            s.implies(la, lb)
            cl.push((na, b))
        elif op == 3:
            s.add_xor(la, lb)
            cl.push((a, b))
            cl.push((na, nb))
        elif op == 4:
            s.add_eq(la, lb)
            cl.push((na, b))
            cl.push((a, nb))
        else:
            let lits = [(rng.int(0, 3), rng.int(0, 1) == 1) for _ in 0..rng.int(0, 3)]
            s.at_most_one([Lit(X(l.0), l.1) for l in lits])
            for i in 0..lits.len:
                for j in i + 1..lits.len:
                    cl.push(((lits[i].0, not lits[i].1), (lits[j].0, not lits[j].1)))
    fn holds(m: Int, l: (Int, Bool)) -> Bool = ((m >> l.0) & 1 == 1) == l.1
    let sat = (0..16).any(m => cl.all(c => holds(m, c.0) or holds(m, c.1)))
    if let got = s.solve():
        let m = (0..4).sum(i => if got[X(i)]: 1 << i else: 0)
        if not cl.all(c => holds(m, c.0) or holds(m, c.1)): bad = "op {k}: TwoSat assignment"
    elif sat: bad = "op {k}: TwoSat missed"
    // de Bruijn: Lyndon words of length dividing `len`, in order, concatenated
    let kk = rng.int(1, 3)
    let len = rng.int(1, 3)
    var words = List<List<Int>>()
    for d in 1..len + 1:
        if len % d == 0:
            var total = 1
            for _ in 0..d: total *= kk
            for code in 0..total:
                var w = List<Int>()
                var c = code
                for _ in 0..d:
                    w.push(c % kk)
                    c /= kk
                reverse(mut w)
                if (1..d).all(r => w < w[r..d].to_list() + w[0..r].to_list()): words.push(w)
    words.sort()
    var want = List<Int>()
    for w in words: want = want + w
    if de_bruijn(kk, len) != want: bad = "op {k}: de_bruijn {kk} {len}"
    if bad != "": break
print(if bad == "": "ok" else: bad)

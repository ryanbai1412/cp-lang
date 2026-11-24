// expect: ok, output "ok"
// input: "1 300\n"
// model: Window
// model: window_fold
// model: Tracked
// model: track
// model: Undoing
// model: undoing
// model: Sweep
use std.rand.{Rng}
use std.window.{Tracked}

domain V = 0..6
domain D = 0..9

const P = 1000003

// Affine maps x -> a*x + b composed left then right: associative, not commutative.
fn compose(f: (Int, Int), g: (Int, Int)) -> (Int, Int) = (f.0 * g.0 % P, (f.1 * g.0 + g.1) % P)

fn find(par: List<Int>, x: Int) -> Int:
    var y = x
    while par[y] != y: y = par[y]
    y

fn chain(fs: List<(Int, Int)>) -> (Int, Int):
    var acc = (1, 0)
    for f in fs: acc = compose(acc, f)
    acc

fn joined(edges: List<(Int, Int)>) -> List<Int>:
    var par = (0..6).collect()
    for (a, b) in edges:
        let (ra, rb) = (find(par, a), find(par, b))
        if ra != rb: par[ra] = rb
    [find(par, x) for x in 0..6]

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var sum_w = Window(+)
var max_w = Window(max)
var aff_w = Window<(Int, Int), ((Int, Int), (Int, Int)) -> (Int, Int)>(compose)
var q: List<Int> = []
var aq: List<(Int, Int)> = []
var cnt = track(List.filled(9, 0), (mut c, d: D) => c[d.ord] += 1, (mut c, d: D) => c[d.ord] -= 1)
var uw = undoing(Dsu<V>(rollback: true), (mut d, e: (V, V)) => d.unite(e.0, e.1))
var eq: List<(Int, Int)> = []
var cb = List.filled(9, 0)
var sw = Sweep<D, Tracked<List<Int>, D>>(track(List.filled(9, 0), (mut c, d: D) => c[d.ord] += 1, (mut c, d: D) => c[d.ord] -= 1))
var bad = ""
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 5)
    if op <= 1 or q.is_empty:
        let x = rng.int(-20, 20)
        let f = (rng.int(1, P - 1), rng.int(0, P - 1))
        sum_w.push(x)
        max_w.push(x)
        aff_w.push(f)
        q.push(x)
        aq.push(f)
    elif op == 2:
        let x = q.remove(0)
        let f = aq.remove(0)
        sum_w.pop(x)
        max_w.pop(x)
        aff_w.pop(f)
    elif op == 3:
        let a = rng.int(0, 5)
        let b = rng.int(0, 5)
        if eq.is_empty or rng.chance(0.6):
            uw.push((V(a), V(b)))
            eq.push((a, b))
        else:
            let e = eq.remove(0)
            uw.pop((V(e.0), V(e.1)))
    elif op == 4:
        let l = rng.int(0, 8)
        let r = rng.int(l, 8)
        if rng.chance(0.2): sw.move(D(l)..D(l))
        else: sw.move(D(l)..=D(r))
    else:
        let xs = [rng.int(-9, 9) for _ in 0..rng.int(0, 8)]
        let w = rng.int(1, 4)
        let got = xs.window_fold(w, max).collect()
        let want = [(i..i + w).map(j => xs[j]).max()! for i in 0..xs.len - w + 1]
        if got != want: bad = "op {k}: window_fold({xs}, {w}, max) = {got} want {want}"
        let gs = xs.window_fold(w, +).collect()
        let ws = [(i..i + w).map(j => xs[j]).sum() for i in 0..xs.len - w + 1]
        if bad == "" and gs != ws: bad = "op {k}: window_fold({xs}, {w}, +) = {gs} want {ws}"
        let d = D(rng.int(0, 8))
        if rng.chance(0.5) or cb[d.ord] == 0:
            cnt.push(d)
            cb[d.ord] += 1
        else:
            cnt.pop(d)
            cb[d.ord] -= 1
    if bad != "": break
    if sum_w.len != q.len or sum_w.is_empty != q.is_empty or max_w.len != q.len:
        bad = "op {k}: len {sum_w.len} want {q.len}"
    elif sum_w.items.collect() != q or aff_w.items.collect() != aq:
        bad = "op {k}: items {sum_w.items.collect()} want {q}"
    elif q.is_empty and (sum_w.all != None or max_w.all != None or aff_w.all != None):
        bad = "op {k}: all of an empty window is not None"
    elif not q.is_empty and sum_w.all! != q.sum():
        bad = "op {k}: sum {sum_w.all!} want {q.sum()}"
    elif not q.is_empty and max_w.all! != q.max()!:
        bad = "op {k}: max {max_w.all!} want {q.max()!}"
    elif not q.is_empty and aff_w.all! != chain(aq):
        bad = "op {k}: affine {aff_w.all!} want {chain(aq)}"
    if bad == "" and cnt.state != cb: bad = "op {k}: tracked {cnt.state} want {cb}"
    let roots = joined(eq)
    for a in 0..6:
        for b in 0..6:
            if bad == "" and uw.state.same(V(a), V(b)) != (roots[a] == roots[b]):
                bad = "op {k}: undoing same({a}, {b}) wrong over {eq}"
    let r = sw.range
    let inside = [if r.contains(D(i)): 1 else: 0 for i in 0..9]
    if bad == "" and sw.state.state != inside: bad = "op {k}: sweep counts {sw.state.state} want {inside}"
    if bad == "" and r.len != inside.sum(): bad = "op {k}: sweep range {r}"
print(if bad == "": "ok" else: bad)

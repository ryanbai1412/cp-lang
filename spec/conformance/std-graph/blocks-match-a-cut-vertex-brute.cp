// expect: ok, output "ok"
// rule: 04.2.1
// `blocks` is a std fn: on random multigraphs with loops and parallel edges, two edges share a
// block exactly when no vertex's removal separates them, and each loop is a block of its own
// numbered after the rest in edge order.
use std.graph.{Graph, blocks}
var seed = 12345
fn next(k: Int) -> Int:
    seed = (seed * 1_103_515_245 + 12_345) % 2_147_483_648
    (seed >> 8) % k
fn find(up: List<Int>, x: Int) -> Int:
    var r = x
    while up[r] != r: r = up[r]
    r
fn apart(eu: List<Int>, ev: List<Int>, n: Int, x: Int, e: Int, f: Int) -> Bool:
    var up = [i for i in 0..n]
    for k in 0..eu.len:
        if eu[k] != x and ev[k] != x:
            let (a, b) = (find(up, eu[k]), find(up, ev[k]))
            up[a] = b
    let a = if eu[e] == x: ev[e] else: eu[e]
    let b = if eu[f] == x: ev[f] else: eu[f]
    find(up, a) != find(up, b)
var ok = true
for trial in 0..300:
    let n = 1 + next(6)
    let m = next(9)
    let eu = [next(n) for _ in 0..m]
    let ev = [next(n) for _ in 0..m]
    domain V = 0..n
    domain E = 0..m
    let es: [E](u: V, v: V) = [E] e => (u: V(eu[e.ord]), v: V(ev[e.ord]))
    let (domain B, block) = blocks(Graph.undirected(es))
    var loops = 0
    for e in E:
        if eu[e.ord] == ev[e.ord]:
            ok = ok and block[e].ord == B.count - (m - e.ord) + (0..m).count(k => k > e.ord and eu[k] != ev[k])
            loops += 1
    for e in E:
        for f in E:
            if eu[e.ord] == ev[e.ord] or eu[f.ord] == ev[f.ord]:
                ok = ok and (block[e] == block[f]) == (e == f)
                continue
            var together = not apart(eu, ev, n, -1, e.ord, f.ord)
            for x in 0..n:
                together = together and not apart(eu, ev, n, x, e.ord, f.ord)
            ok = ok and (block[e] == block[f]) == together
print(if ok: "ok" else: "bad")

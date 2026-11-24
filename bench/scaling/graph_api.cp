// n: 65536
// fixed: k, deg, u
use std.graph.{Graph, Lit, arc_len, bfs, complement, euler_walk}
// O(1) accessors and per-arc iterators of graph types, on a graph whose every vertex has
// out-degree 2 (arcs i -> i+1 and i -> i+3 mod n), so a whole `out(u)` is O(1).
let n: Int = read()
domain V = 0..n
domain E = 0..2 * n
let es: [E](u: V, v: V) = [E] e => (u: V(e.ord % n), v: V((e.ord % n + 1 + 2 * (e.ord / n)) % n))
let wes: [E](u: V, v: V, w: Int) = [E] e => (u: es[e].u, v: es[e].v, w: e.ord % 10 + 1)
let cyc = [V] v => [V((v.ord + 1) % n), V((v.ord + 3) % n)]
let gd = Graph.directed(es)
let gw = Graph.directed(wes)
let p = bfs(gd, from: [V(0)])
let walk = euler_walk(es, directed: true)!
// Views made once: making one copies the graph (a codegen copy of a live argument), so they aren't timed here.
let o = gd.only(keep: x => x.ord % 2 == 0)
let rw = gd.weighted(a => a.e.ord % 3)
var s = 0
for j in 0..2 * n:
    let i = j % n
    let u = V(i)
    let v = V((i * 7919) % n)
    s += gd.at(u).len // timed: graph.Graph.at
    s += gd.out(u).count() // timed: graph.Graph.out
    s += gd.deg(u) // timed: graph.Graph.deg
    s += gd.next(u).count() // timed: graph.Graph.next
    s += Int(gd.contains(u, v)) // timed: graph.Graph.contains
    s += o.out(u).count() // timed: graph.Only.out
    s += o.deg(u) // timed: graph.Only.deg
    s += rw.out(u).count() // timed: graph.Reweighted.out
    s += cyc.out(u).count() // timed: graph.Next.out
    s += cyc.deg(u) // timed: graph.deg
    s += cyc.next(u).count() // timed: graph.next
    let a = gw.out(u).first()!
    s += a.w // timed: graph.Weighted.w
    s += a.e.ord // timed: graph.Edged.e
    s += a.to.ord // timed: graph.Target.to
    s += arc_len(a) // timed: graph.arc_len
    s += Int(Lit(u, true).neg.value) // timed: graph.Lit.neg
    s += p.get(u) ?? 0 // timed: graph.Paths.get
    s += Int(p.reached(u)) // timed: graph.Paths.reached
    var seen = 0
    for _ in p.iter(): // timed: graph.Paths.iter
        seen += 1
        if seen == 8: break
    for _ in walk.vertices: // timed: graph.Walk.vertices
        seen += 1
        if seen == 16: break
    s += seen
let comp = complement(gd)
for i in 0..64:
    s += comp.out(V(i)).count() // timed: graph.Complement.out
    s += gd.edges().count() // timed: graph.Graph.edges
print(s)

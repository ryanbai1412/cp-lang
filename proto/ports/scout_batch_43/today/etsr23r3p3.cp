// Runnable port (proto/cp.py lib-run) of etsr23r3p3.cp
// original: corpus/submissions/5521298.cpp
use std.graph.{Graph, dijkstra}
use std.dsu.{Dsu}
use std.tree.{Rooted}

const INF = 2147483647

let tcs: Int = read()
for _ in 0..tcs:
    let (n, m): (Int, Int) = read()
    domain V = 1..=n
    domain E = 0..m
    var eu: List<V> = []
    var ev: List<V> = []
    var el: List<Int> = []
    var ew: List<Int> = []
    for _ in 0..m:
        let (u, v, l, w): (Int, Int, Int, Int) = read()
        eu.push(V(u))
        ev.push(V(v))
        el.push(l)
        ew.push(w)

    let g = Graph<V, E>.undirected([E] e => (u: eu[e.to_idx], v: ev[e.to_idx]))
    let dist = dijkstra::<V>(u => g.adj(u), [(V.first, 0)], a => el[a.e.to_idx])

    let order_keys = sorted([(-ew[i], i) for i in 0..m])

    var dsu = Dsu<V>()
    var root_of: [V]Int = [V] v => v.to_idx
    var kids: List<List<Int>> = [[] for _ in 0..n]
    var gate: List<?Int> = [none for _ in 0..n]
    var mn: List<Int> = [if dist[v] == none: INF else: dist[v]! for v in V]

    for item in order_keys:
        let ei = item.1
        let ru = dsu.find(eu[ei])
        let rv = dsu.find(ev[ei])
        if ru != rv:
            let a = root_of[ru]
            let b = root_of[rv]
            let k = kids.len
            kids.push([a, b])
            gate.push(ew[ei])
            mn.push(min(mn[a], mn[b]))
            dsu.unite(eu[ei], ev[ei])
            root_of[dsu.find(eu[ei])] = k

    let total = kids.len
    domain K = 0..total
    let ktree = Rooted<K>(u => [K(c) for c in kids[u.to_idx]], root: K.last)

    let (q, online, tmax): (Int, Int, Int) = read()
    var last_ans = 0
    for _ in 0..q:
        let (u0, t0): (Int, Int) = read()
        let u = K((u0 + online * last_ans - 1) % n)
        let t = (t0 + online * last_ans) % (tmax + 1)
        let top = ktree.lift_while(u, p => if gate[p.to_idx] == none: true else: gate[p.to_idx]! > t)!
        last_ans = mn[top.to_idx]
        print(last_ans)

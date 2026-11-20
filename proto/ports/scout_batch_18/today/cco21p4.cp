// Runnable port (proto/cp.py lib-run) of statement-first cco21p4.cp
// original: corpus/submissions/6419583.cpp

let (n, m): (Int, Int) = read()
domain V = 1..=n
domain E = 0..m
let roads: [E](a: V, b: V, r: Int, p: Int) = read()

let rev = Graph<V, E>.directed([E] e => (u: roads[e].b, v: roads[e].a))
var out_deg: [V]Int = [V] _ => 0
for e in E: out_deg[roads[e].a] += 1

var ans: [V]?Int = [V] _ => none
var removed: [E]Bool = [E] _ => false
var dead: [V]Bool = [V] _ => false
var peel: List<V> = []
for v in V:
    if out_deg[v] == 0: peel.push(v)
var head = 0

fn drain_peel():
    while head < peel.len:
        let v = peel[head]
        head += 1
        dead[v] = true
        for arc in rev.adj(v):
            let e = arc.e
            if removed[e]: continue
            removed[e] = true
            let u = roads[e].a
            if let av = ans[v]:
                let need = max(roads[e].r, av - roads[e].p)
                if let cur = ans[u]:
                    if need < cur: ans[u] = need
                else:
                    ans[u] = need
            out_deg[u] -= 1
            if out_deg[u] == 0 and not dead[u]:
                peel.push(u)

drain_peel()
let order = sorted([(-roads[e].r, e.to_idx) for e in E])
for pair in order:
    let e = E(pair.1)
    if removed[e]: continue
    removed[e] = true
    let u = roads[e].a
    if let cur = ans[u]:
        if roads[e].r < cur: ans[u] = roads[e].r
    else:
        ans[u] = roads[e].r
    out_deg[u] -= 1
    if out_deg[u] == 0:
        peel.push(u)
        drain_peel()

for v in V:
    print(ans[v] ?? -1)

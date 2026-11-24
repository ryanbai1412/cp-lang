// Source: CCO 2021 P4 (Travelling Merchant), https://dmoj.ca/problem/cco21p4
// Problem: n, m <= 2e5; one-way road i from a to b needs at least r_i money and then pays
//   p_i >= 0. For every starting city print the least starting money that lets the merchant
//   travel forever, or -1.
// Solution: a city with no usable outgoing road is dead: peel dead cities backwards and
//   relax ans[u] = min(ans[u], max(r, ans[v] - p)) over their incoming roads. Among the
//   remaining roads the one with the largest r gives its start city r (every road from
//   there on is affordable); record it, delete the road and peel anew. O((n + m) log m).
// original: corpus/submissions/6419583.cpp

let (n, m): (Int, Int) = read()
domain V = 1..=n
domain E = 0..m
let roads: [E](a: V, b: V, r: Int, p: Int) = read()

let rev = Graph<V, E>.directed([E] e => (u: roads[e].b, v: roads[e].a))
var out_deg: [V]Int = [V] _ => 0
for e in E: out_deg[roads[e].a] += 1

var ans: [V]?Int = [V] _ => None
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
        for arc in rev.out(v):
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
let order = sorted([(-roads[e].r, e.ord) for e in E])
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

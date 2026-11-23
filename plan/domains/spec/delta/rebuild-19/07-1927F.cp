// CF 1927F Microcycle: add edges heaviest first; the last edge that closes a cycle is the
// answer's lightest edge, and the cycle is it plus any path through no lighter edge.
@!multitest
let n: 3..=2e5 = read()
let m: 3..=2e5 = read()
domain V = 1..=n
let es = read<[m](u: V, v: V, w: 1..=1e6)>()
var d = Dsu<V>()
var best: ?Int = None
for e in es.keys.sorted(by: e => es[e].w, desc: true):
    if not d.unite(es[e].u, es[e].v): best = Some(e)
let b = best!
var heavy: [V]List<V> = [V] _ => []
for e in es.keys:
    if e != b and es[e].w >= es[b].w:
        heavy[es[e].u].push(es[e].v)
        heavy[es[e].v].push(es[e].u)
let cycle = bfs(heavy, from: [es[b].u], until: es[b].v).path(es[b].v)!
print(es[b].w, cycle.len)
print(cycle)

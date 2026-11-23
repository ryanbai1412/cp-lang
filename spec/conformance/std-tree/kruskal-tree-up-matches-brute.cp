// expect: ok, output "YES"
var ok = true
for trial in 0..30:
    domain V = 0..8
    let es: [0..10](u: V, v: V, w: Int) = [0..10] _ => (u: V(rand(0, 7)), v: V(rand(0, 7)), w: rand(1, 6))
    let val: [V]Int = [V] _ => rand(-5, 9)
    let (domain N, kt) = kruskal_tree(es, w: e => es[e].w)
    let tot = kt.up(+, leaf: v => val[v])
    let low = kt.up(min, leaf: v => val[v])
    for lim in 0..=7:
        var d = Dsu<V>()
        for e in es.keys:
            if es[e].w <= lim: d.unite(es[e].u, es[e].v)
        for v in V:
            let x = kt.reach(v, lim)
            let same = [u for u in V if d.same(u, v)]
            if tot[x] != sum([val[u] for u in same]): ok = false
            if low[x] != [val[u] for u in same].min(): ok = false
print(if ok: "YES" else: "NO")

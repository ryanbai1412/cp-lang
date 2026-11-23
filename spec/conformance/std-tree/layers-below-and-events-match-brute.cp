// expect: ok, output "YES"
use std.graph.{Visit}
var ok = true
for trial in 0..30:
    domain V = 0..20
    var adj: [V]List<V> = []
    for i in 1..20:
        let p = if trial % 2 == 0: rand(0, i - 1) else: rand(max(0, i - 3), i - 1)
        adj[V(i)].push(V(p))
        adj[V(p)].push(V(i))
    let t = Rooted(adj, root: V(rand(0, 19)))
    let ly = t.layers()
    let ev = t.events()
    let tour = t.tour.collect()
    for v in V:
        for d in -1..=21:
            var want: List<V> = []
            for p in t.sub(v):
                if t.depth(t.at[p]) == d: want.push(t.at[p])
            if ly.below(v, d).map(b => ly.at[b]).collect() != want: ok = false
        if ev.at[ev.enter(v)] != Visit.Enter(v, t.parent(v)): ok = false
        if ev.at[ev.exit(v)] != Visit.Exit(v, t.parent(v)): ok = false
        if ev.sub(v).count() != 2 * t.size(v): ok = false
        if ev.at[ev.sub(v).first!] != Visit.Enter(v, t.parent(v)): ok = false
        for w in V:
            let pa = ev.path(v, w)
            var odd: [V]Bool = false
            for p in pa.range:
                let u = match ev.at[p]:
                    Enter(u, _) => u
                    Exit(u, _) => u
                    Again(u, _) => u
                odd[u] = not odd[u]
            if let c = pa.extra: odd[c] = not odd[c]
            for u in V:
                if odd[u] != t.on_path(u, v, w): ok = false
print(if ok: "YES" else: "NO")

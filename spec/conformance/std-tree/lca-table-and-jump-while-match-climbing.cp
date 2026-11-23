// expect: ok, output "YES"
fn climb<V>(par: [V]?V, v: V) -> List<V>:
    var out: List<V> = [v]
    while let p = par[out[out.len - 1]]: out.push(p)
    out
var ok = true
for trial in 0..30:
    domain V = 0..25
    var adj: [V]List<V> = []
    for i in 1..25:
        let p = if trial % 2 == 0: rand(0, i - 1) else: rand(max(0, i - 3), i - 1)
        adj[V(i)].push(V(p))
        adj[V(p)].push(V(i))
    let root = V(rand(0, 24))
    let t = Rooted(adj, root: root)
    let lt = LcaTable(t)
    let par = [V] v => t.parent(v)
    for a in V:
        let ca = climb(par, a)
        for b in V:
            let cb = climb(par, b)
            var want = root
            for u in ca: if cb.any(x => x == u) and t.depth(u) > t.depth(want): want = u
            if lt.lca(a, b) != want or lt.lca(a, b) != t.lca(a, b): ok = false
            if lt.dist(a, b) != t.dist(a, b): ok = false
        for limit in 0..=25:
            var want: ?V = None
            for u in ca:
                if t.depth(u) >= limit: want = Some(u) else: break
            if t.jump_while(a, u => t.depth(u) >= limit) != want: ok = false
            var near: ?V = None
            for u in ca:
                if t.dist(u, a) <= limit: near = Some(u) else: break
            if t.jump_while(a, u => t.dist(u, a) <= limit) != near: ok = false
print(if ok: "YES" else: "NO")

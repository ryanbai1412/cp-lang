// expect: ok, output "YES"
var ok = true
for trial in 0..20:
    domain V = 0..15
    var adj: [V]List<V> = []
    for i in 1..15:
        let p = if trial % 2 == 0: rand(0, i - 1) else: rand(max(0, i - 3), i - 1)
        adj[V(i)].push(V(p))
        adj[V(p)].push(V(i))
    let t = Rooted(adj, root: V(rand(0, 14)))
    for root in V:
        let u = Rooted(adj, root: root)
        for v in V:
            var got: [V]Bool = false
            var pieces = 0
            for r in t.sub_at(v, root):
                pieces += 1
                if r.is_empty: ok = false
                for p in r: got[t.at[p]] = true
            if pieces < 1 or pieces > 2: ok = false
            for w in V:
                if got[w] != u.is_anc(v, w): ok = false
print(if ok: "YES" else: "NO")

// expect: ok, output "YES"
use std.graph.{Graph}
use std.tree.{spanning_tree}
var ok = true
for trial in 0..30:
    domain V = 0..9
    var es: List<(u: V, v: V)> = []
    for i in 1..9: es.push((u: V(rand(0, i - 1)), v: V(i)))
    for _ in 0..rand(0, 6): es.push((u: V(rand(0, 8)), v: V(rand(0, 8))))
    shuffle(mut es)
    domain E = 0..es.len
    let arr: [E](u: V, v: V) = [E] e => es[e.to_idx]
    let g = Graph.undirected(arr)
    let t = g.spanning_tree()
    var adj: [V]List<V> = []
    for e in E:
        adj[arr[e].u].push(arr[e].v)
        if arr[e].u != arr[e].v: adj[arr[e].v].push(arr[e].u)
    var dist: [V]?Int = None
    var par: [V]?V = None
    dist[V(0)] = Some(0)
    var q = [V(0)]
    var h = 0
    while h < q.len:
        let u = q[h]
        h += 1
        for w in adj[u]:
            if dist[w] == None:
                dist[w] = Some(dist[u]! + 1)
                par[w] = Some(u)
                q.push(w)
    if t.root != V(0): ok = false
    for v in V:
        if t.depth(v) != dist[v]!: ok = false
        if t.parent(v) != par[v]: ok = false
print(if ok: "YES" else: "NO")

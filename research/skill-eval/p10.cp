let n = read<1..=2e5>()
domain V = 1..=n
let edges = read<Tree<V>>()
let g = Graph.undirected(edges)
let t = Rooted(g, root: V(1))
var matched: [V]Bool = false
var ans = 0
for p in t.Pre.rev:
    let v = t.at[p]
    if let par = t.parent(v):
        if not matched[v] and not matched[par]:
            matched[v] = true
            matched[par] = true
            ans += 1
print(ans)

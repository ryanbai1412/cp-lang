let n = read<1..=2e5>()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V(1))
var matched = [V] v => false
var pairs = 0
for p in t.Pre.rev:
    let v = t.at[p]
    if let Some(u) = t.parent(v):
        if not matched[v] and not matched[u]:
            matched[v] = true
            matched[u] = true
            pairs += 1
print(pairs)

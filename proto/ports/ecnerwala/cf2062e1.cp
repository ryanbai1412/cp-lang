// Codeforces 2062 E1: The Game (Easy Version).
// Scan nodes in decreasing weight order, maintaining the LCA of all nodes
// strictly heavier than the current weight. The first node v that is not an
// ancestor of that LCA has a heavier node outside its subtree, while every
// heavier node contains all nodes heavier than itself — so picking v wins.
@!multitest
let n: 1..=400_000 = read()
domain V = 1..=n
domain W = 1..=n
let w: [V]W = read()
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted.new(Graph.undirected(edges), V.first)

var by_w: [W]List<V> = []
for v in V: by_w[w[v]].push(v)

fn win() -> ?V:
    var heavier_lca: ?V = none
    for weight in W.rev:
        if let l = heavier_lca:
            if let v = first(by_w[weight], v => not t.is_anc(v, l)): return v
        for v in by_w[weight]:
            heavier_lca = if let l = heavier_lca: t.lca(l, v) else: v
    none

if let v = win(): print(v) else: print(0)

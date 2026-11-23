// Codeforces 2062 E1: The Game (Easy Version) (dream).
// Scan nodes in decreasing weight order, maintaining the LCA of all nodes
// strictly heavier than the current weight. The first node v that is not an
// ancestor of that LCA has a heavier node outside its subtree, while every
// heavier node contains all nodes heavier than itself — so picking v wins.
with io(base: 1)

@!multitest
domain V = read(1..=4e5)
let w: [V]1..=V.count = read()
let t = Rooted(Graph.undirected(read<[V.count - 1](V, V)>()), root: V.first)

let by_w = V.group(by: v => w[v])

fn win() -> ?V:
    var heavier_lca: ?V = None
    for bucket in by_w.rev:
        if let l = heavier_lca:
            if let v = bucket.first(v => not t.is_anc(v, l)): return v
        for v in bucket:
            heavier_lca = if let l = heavier_lca: t.lca(l, v) else: v
    None

print(win() ?? 0)

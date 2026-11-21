// Codeforces 2062 E1: The Game (Easy Version).
// Scan nodes in decreasing weight order, maintaining the LCA of all nodes
// strictly heavier than the current weight. The first node v that is not an
// ancestor of that LCA has a heavier node outside its subtree, while every
// heavier node contains all nodes heavier than itself — so picking v wins.
use std.seq.{buckets}

@!multitest
let n: 1..=400_000 = read()
domain V = 1..=n
domain W = 1..=n
let w: [V]W = read()
let t = Rooted(Graph.undirected(read::<[0..n - 1](V, V)>()), V.first)
let by_w = buckets(w)

fn win() -> ?V:
    var heavier_lca: ?V = none
    for weight in W.rev:
        let b = by_w[weight]
        if let l = heavier_lca:
            if let v = first(b, v => not t.is_anc(v, l)): return v
        for v in b:
            heavier_lca = if let l = heavier_lca: t.lca(l, v) else: v
    none

if let v = win(): print(v) else: print(0)

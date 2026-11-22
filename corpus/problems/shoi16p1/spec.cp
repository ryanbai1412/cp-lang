// SHOI 2016 "Reconnaissance Guards": a guard at v (cost c[v]) watches every
// node within k; watch all required nodes as cheaply as possible. Tree DP with
// a signed reach r: r >= 0, the subtree is watched and a guard's view reaches r
// more steps above v; r < 0, unwatched required nodes sit at most -r-1 below v.
let (n, k) = read<(1..=5e5, 1..=20)>()
domain V = 1..=n
let cost = read<[V]1..=1000>()
let must = V.mask(read<List<V>>())
let t = Rooted(Graph<V>.undirected(read<[n - 1](u: V, v: V)>()), root: V(1))
domain Reach = -k..=k

var best: [V][Reach]Int = []
for v in List(t.subtree(t.root)).rev:
    let kids = t.children(v).map(c => best[c]).collect()
    var b = [Reach] r =>
        if r == k: cost[v] + kids.sum(of: c => c[-k])
        elif r >= 0: kids.sum(of: c => c[-r]) + (kids.min(of: c => c[r + 1] - c[-r]) ?? inf)
        else: kids.sum(of: c => c[r + 1])
    if not must[v]:
        b[Reach(0)] = min(b[Reach(0)], kids.sum(of: c => c[Reach(0)]))
    best[v] = b.suffix_min()
print(best[t.root][0])

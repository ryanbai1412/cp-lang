// SHOI 2016 "Reconnaissance Guards": a guard at v (cost c[v]) watches every node
// within k; watch all required nodes as cheaply as possible. Tree DP with a
// signed reach r: r >= 0, the subtree is watched and a guard's view reaches r
// more steps above v; r < 0, unwatched required nodes sit at most -r-1 below v.
// best[v] is a suffix min: more reach is never worse.
let (n, k) = read<(1..=5e5, 1..=20)>()
domain V = 1..=n
let cost = read<[V]1..=1000>()
let m = read<1..=n>()
let must = V.mask(read<[m]V>())
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V.first!)
domain Reach = -k..=k

var best: [V][Reach]Int = 0
for v in t.order.rev:
    let kids = t.children(v)
    let need = (r: Int) => kids.sum(of: c => best[c][Reach(r)])
    var b = [Reach] r =>
        let d = Int(r)
        if d == k: cost[v] + need(-k)
        elif d >= 0: need(-d) + (kids.map(c => best[c][r.next!] - best[c][Reach(-d)]).min() ?? inf)
        else: need(d + 1)
    if not must[v]: b[Reach(0)] min= need(0)
    for r in Reach.rev.drop(1): b[r] min= b[r.next!]
    best[v] = b
print(best[t.root][Reach(0)])

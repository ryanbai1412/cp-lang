// SHOI 2016 "Reconnaissance Guards": a guard at v (cost c[v]) watches every
// node within k; watch all required nodes as cheaply as possible. Tree DP with
// a signed reach r: r >= 0, the subtree is watched and a guard's view reaches r
// more steps above v; r < 0, unwatched required nodes sit at most -r-1 below v.
let n: 1..=5e5 = read()
let k: 1..=20 = read()
domain V = 1..=n
let cost = read<[V]1..=1000>()
let m: 1..=n = read()
let must = V.mask(read<[m]V>())
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V.first!)
domain Reach = -k..=k
var best: [V, Reach]Int = 0
for v in t.order.rev:
    let kids = t.children(v).collect()
    fn at(r: Reach) -> Int = sum(best[c, r] for c in kids)
    var b: [Reach]?Int = [Reach] r =>
        if r == Reach.last!: cost[v] + at(Reach.first!)
        elif r.label >= 0: min(
            best[c, r.next!] - best[c, Reach(-r.label)] for c in kids
        ) + at(Reach(-r.label))
        else: at(r.next!)
    if not must[v]: b[Reach(0)] min= at(Reach(0))
    for r in Reach.rev:
        if let up = r.next: b[r] min= b[up]
        best[v, r] = b[r]!
print(best[t.root, Reach(0)])

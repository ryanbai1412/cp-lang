// cf1988D, revised: print, for every vertex v, the least damage if only the monsters of
// v's subtree (rooted at 1) were on the board.
@!multitest
let n: 1..=3e5 = read()
domain V = 1..=n
let a = read<[V]1..=1e12>()
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V.first!)
domain Round = 1..=20

// cost[v, r]: least damage from v's subtree when v dies in round r
var cost: [V, Round]Int = 0
for v in t.order.rev:
    for r in Round: cost[v, r] += Int(r) * a[v]
    for c in t.children(v):
        let best = Round.min(by: q => cost[c, q])!
        let second = Round.filter(q => q != best).map(q => cost[c, q]).min()!
        for r in Round: cost[v, r] += if r == best: second else: cost[c, best]
print(V.map(v => Round.map(r => cost[v, r]).min()!))

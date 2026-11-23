// CF 1988D: a monster killed in round c costs a * c; neighbours differ in round.
// Tree DP over rounds 1..=20 (an optimal colouring never needs more).
@!multitest
let n: 1..=3e5 = read()
domain V = 1..=n
let a = read<[V]1..=1e12>()
let edges = read<[n - 1](u: V, v: V)>()
let t = Rooted(Graph.undirected(edges), root: V.first!)

domain Round = 1..=20
var cost: [V, Round]Int = 0
for v in t.order.rev:
    for c in Round: cost[v, c] += a[v] * Int(c)
    if let Some(p) = t.parent(v):
        let best = Round.min(by: c => cost[v, c])!
        let second = Round.filter(c => c != best).map(c => cost[v, c]).min()!
        for c in Round: cost[p, c] += if c == best: second else: cost[v, best]
print(Round.map(c => cost[V.first!, c]).min()!)

// cf1988D The Omnipotent Monster Killer: kill each vertex in a round 1..=20 so that
// neighbours differ; a monster killed in round r attacks r times. 20 rounds suffice
// (log2(3e5) + 2), and the total stays below 3e5 * 20 * 1e12 = 6e18 < 2^63.
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
print(Round.map(r => cost[V.first!, r]).min()!)

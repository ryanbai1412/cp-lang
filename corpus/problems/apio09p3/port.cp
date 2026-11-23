// APIO 2009 ATM: one-way roads, an ATM with some cash at every
// intersection, robbed at most once however often it is passed. Start at
// `start`, end at any pub, take as much as possible.
//
// Inside a strongly connected component everything can be taken, so the
// components form a DAG with a loot each, and the answer is its best path
// to a component with a pub.

let (n, m): (Int, Int) = read()
domain Spot = 1..=n
domain Road = 0..m
let roads: [Road](u: Spot, v: Spot) = read()
let cash: [Spot]Int = read()
let (start, p): (Spot, Int) = read()
domain P = 0..p
let pubs: [P]Spot = read()

let g = Graph<Spot, Road>.directed(roads)
let (C, comp) = scc::<Spot>(u => g.out(u))
let onward = condense::<Spot>(u => g.out(u), comp)

var loot: [C]Int = 0
for v in Spot: loot[comp[v]] += cash[v]
var has_pub: [C]Bool = false
for v in pubs: has_pub[comp[v]] = true

// best[c]: the most a walk can take from c on, ending at a pub; none if it
// cannot end at one. Roads lead to later components, so fill from the back.
var best: [C]?Int = None
for c in C.rev:
    var after: ?Int = if has_pub[c]: 0 else: None
    for a in onward[c]:
        if let b = best[a.to]: after = max(after ?? b, b)
    if let x = after: best[c] = loot[c] + x
print(best[comp[start]]!)

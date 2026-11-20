// APIO 2009 ATM: one-way roads between junctions, an ATM with some cash at
// each, pubs at some. Start at `start`, rob every ATM passed (once each),
// revisit roads freely, and end at a pub. Maximise the cash.
let (n, m): (1..=5e5, 1..=5e5) = read()
domain Junction = 1..=n
domain Road = 0..m
let roads: [Road](u: Junction, v: Junction) = read()
let cash: [Junction]0..=4000 = read()
let (start, p): (Junction, 1..=5e5) = read()
let pubs: [0..p]Junction = read()

// Inside a strongly connected district everything can be robbed, so shrink
// each district to a point: the rest is a longest path in a DAG.
let city = Graph.directed(roads)
let (District, of) = scc(city)
var loot: [District]Int = 0
for j in Junction: loot[of[j]] += cash[j]
var best: [District]?Int = none             // most cash from here on, ending at a pub
for j in pubs: best[of[j]] = 0
let onward = condense(city, of)
for d in District.rev:                      // roads lead to later districts
    for e in onward[d]: best[d] max= best[e.to]
    best[d] = best[d] + loot[d]
print(best[of[start]]!)

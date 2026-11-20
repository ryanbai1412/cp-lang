// cp: 0.1 0f8d77b+dirty
// APIO 2009 ATM: one-way roads between junctions, an ATM with some cash at
// each, pubs at some. Start at `start`, rob every ATM passed (once each),
// revisit roads freely, and end at a pub. Maximise the cash.
let n: 1..=5e5 = read()
let m: 1..=5e5 = read()
domain Junction = 1..=n
let roads: [m](u: Junction, v: Junction) = read()
let cash: [Junction]0..=4000 = read()
let start: Junction = read()
let p: 1..=n = read()
domain Pub = 0..p

fn first_at_start() -> [Pub]Junction:       // generator: some pub is reachable
    [Pub] i => (if i == Pub.first: start else: Junction(rand(1, n)))

let pubs: [Pub]Junction = read(gen: first_at_start)

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
    for e in onward[d]:
        if let b = best[e]: best[d] max= b
    if let b = best[d]: best[d] = b + loot[d]
print(best[of[start]]!)

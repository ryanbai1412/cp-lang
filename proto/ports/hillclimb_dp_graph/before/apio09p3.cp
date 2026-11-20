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

fn first_at_start() -> [Pub]Junction:  // generator: some pub is reachable
    [Pub] i => (if i == Pub.first: start else: Junction(rand(1, n)))

let pubs: [Pub]Junction = read(gen: first_at_start)

// Inside a strongly connected district everything can be robbed, so shrink
// each district to a point: the rest is a longest path in a DAG.
var out: [Junction]List<Junction> = []
var back: [Junction]List<Junction> = []
for (u, v) in roads:
    out[u].push(v)
    back[v].push(u)

// Kosaraju: finish order on the roads, then sweep the reversed roads
var seen: [Junction]Bool = false
var finished: List<Junction> = []
fn finish(u: Junction):
    seen[u] = true
    for v in out[u]:
        if not seen[v]: finish(v)
    finished.push(u)
for j in Junction:
    if not seen[j]: finish(j)

var district: [Junction]Int = -1  // numbered so that roads lead to later districts
var districts = 0
fn claim(u: Junction):
    district[u] = districts
    for v in back[u]:
        if district[v] < 0: claim(v)
for j in finished.rev:
    if district[j] < 0:
        claim(j)
        districts += 1

domain District = 0..districts
let of = [Junction] j => District(district[j])
var loot: [District]Int = 0
for j in Junction: loot[of[j]] += cash[j]
var best: [District]?Int = none  // most cash from here on, ending at a pub
for j in pubs: best[of[j]] = 0
var members: [District]List<Junction> = []
for j in Junction: members[of[j]].push(j)
for d in District.rev:  // roads lead to later districts
    for j in members[d]:
        for v in out[j]:
            if of[v] != d:
                if let b = best[of[v]]: best[d] max= b
    if let b = best[d]: best[d] = b + loot[d]
print(best[of[start]]!)

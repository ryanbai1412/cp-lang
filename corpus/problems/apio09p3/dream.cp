// APIO 2009 ATM: rob every ATM on some route from the start to a pub; maximize the cash.
// Inside a strongly connected component every ATM is reachable, so condense and take the best
// DAG path from the start's component to any component with a pub.
let (n, m): (Int, Int) = read()
domain V = 1..=n
let roads: [m](u: V, v: V) = read()
let cash: [V]Int = read()
let (start, p): (V, Int) = read()
let pubs: [p]V = read()

let g = Graph.directed(roads)
let (C, comp) = scc(g)
let dag = condense(g, comp)
var gain: [C]Int = 0
for v in V: gain[comp[v]] += cash[v]
let ends = Set(comp[v] for v in pubs)

// best[c]: the most cash on a path from c to a pub, None when no pub is reachable.
// Every arc leads to a later component, so fill from the back.
var best: [C]?Int = None
for c in C.rev:
    let onward = dag[c].filter_map(d => best[d.to]).max()
    let tail = if c in ends: Some(onward ?? 0) else: onward
    best[c] = tail.map(t => gain[c] + t)
print(best[comp[start]]!)

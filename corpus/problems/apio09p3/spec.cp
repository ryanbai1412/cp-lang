// apio09p3 ATM: the most cash on a walk from the start that ends at a pub. A walk can loop
// inside a strongly connected component, so the answer is a longest path on the condensation.
let (n, m) = read<(1..=500_000, 0..=500_000)>()
domain V = 1..=n
let g = Graph.directed(read<[m](u: V, v: V)>())
let cash: [V]0..=4000 = read()
let (start, k) = read<(V, 1..=n)>()
let pubs = read<[k]V>()

let (domain C, comp) = scc(g)
let dag = condense(g, comp)
var total: [C]Int = 0
var bar: [C]Bool = false
for v in V:
    total[comp[v]] += cash[v]
for p in pubs:
    bar[comp[p]] = true

// best[c]: the most cash from c to a pub, None when no pub is reachable. Arcs lead to later
// components, so fill from the back. Cash is never negative, so going on to a pub always
// beats stopping at one in c.
var best: [C]?Int = None
for c in C.rev:
    let stop = if bar[c]: Some(0) else: None
    let tail = dag[c].filter_map(a => best[a.to]).max() ?? stop
    best[c] = tail.map(t => total[c] + t)
print(best[comp[start]]!)

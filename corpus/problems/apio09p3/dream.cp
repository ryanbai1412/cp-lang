// apio09p3 ATM: the most cash on a walk from the start that ends at a pub. A walk can loop
// inside a strongly connected component, so the answer is a longest path on the condensation.
let (n, m) = read<(1..=500_000, 1..=500_000)>()
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

@memo(array)
fn best(c: C) -> ?Int:
    var tail = dag[c].filter_map(a => best(a.to)).max()
    if bar[c]:
        tail = max(tail ?? 0, 0)
    tail.map(t => total[c] + t)
print(best(comp[start])!)

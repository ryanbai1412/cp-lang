# APIO 2009 ATM: rob every ATM on some route from the start to a pub; maximize the cash.
# Inside a strongly connected component every ATM is reachable, so condense and take the best
# DAG path from the start's component to any component with a pub.
domain V = [read Int]
let roads = Graph<V>.directed(read List<(u: V, v: V)>)
let cash = read [V]Int
let (start, pubs) = read (V, List<V>)

let (domain C, comp) = roads.sccs()
let dag = roads.condense(comp)
let gain = [C] c => comp.members(c).sum(of: v => cash[v])
let ends = C.mask(pubs.map(p => comp[p]))

# best[c]: the most cash on a path from c to a pub, none when no pub is reachable.
let best = dag.up(order: dag.topo().rev, (c, next) =>
    let tail = max(next.map(d => best[d]).some(), none: if ends[c]: Some(0) else: None)
    tail.map(t => gain[c] + t))
print(best[comp[start]]!)

// p10 tuned: the reference's layout. Compressed rows (`Graph`), a BFS order, and
// the children folded into their parent in reverse order, all in flat arrays.
let n: 1..=2e5 = read()
domain V = 1..=n
let edges = read<[n - 1](u: V, v: V)>()
let g = Graph.undirected(edges, keep: a => (to: a.to))
var par: [V]?V = None
var order: List<V> = [V.first!]
order.reserve(n)
var i = 0
while i < order.len:
    let u = order[i]
    i += 1
    for v in g.next(u) if Some(v) != par[u]:
        par[v] = u
        order.push(v)
var sum: [V]Int = 0
var gain: [V]Int = -inf
var best = 0
for v in order.rev:
    let free = sum[v]
    let used = if gain[v] == -inf: 0 else: free + gain[v]
    best = max(free, used)
    if let p = par[v]:
        sum[p] += best
        gain[p] max= free + 1 - best
print(best)

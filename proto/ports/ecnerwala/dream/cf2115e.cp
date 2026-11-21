// Codeforces 2115 E: Gellyfish and Mayflower.
// On a fixed path, let p be the trader with the best ratio w / c. Any other
// purchase of at least c_p cards has a non-empty sub-multiset whose cost is a
// multiple of c_p (pigeonhole on prefix sums mod c_p); swapping it for p's
// cards loses nothing. So the purchases other than p cost under C * c_p <= C^2.
// - Budgets under cap = C^2: an unbounded knapsack carried along the DAG,
//   small[v, j] = most power on a path to v spending at most j coins.
// - Budgets of at least cap: try each trader as the pivot p. Edges jumping over
//   p are dropped, so a path reaches a vertex >= p only through p, and only
//   traders no better than p are bought from. best[v, j] = most power of the
//   other purchases with cost = j mod c_p, minus w_p per c_p coins they use,
//   and the answer is best[at, r % c_p] + (r / c_p) * w_p.
let (n, m): (1..=200, 1..=2000) = read()
domain V = 1..=n
let item: [V](cost: 1..=200, worth: 1..=1e9) = read()
let arcs: [0..m](u: V, v: V) = read()
let next = Graph.directed(arcs)
let q: 1..=200_000 = read()
let queries: [0..q](at: V, budget: 1..=1e9) = read()

let cap = item.max(by: i => i.cost)!.cost ** 2
var small: [V, 0..=cap]Int = 0
for v in V:
    small.row(v).add_unbounded(item[v])
    for k in next[v]: small.row(k) max= small.row(v)
var ans = [small[x.at, min(x.budget, cap)] for x in queries]

for p in V:
    let (pc, pw) = item[p]
    let through = next.reachable(from: V.first, skip: (u, v) => u < p and p < v)
    var best: [V, 0..pc]Int = 0
    for v in V if through[v]:
        if item[v].worth * pc <= pw * item[v].cost:
            best.row(v).add_unbounded(item[v], wrap: (pc, pw))
        for k in next[v] if not (v < p and p < k): best.row(k) max= best.row(v)
    for (i, x) in queries.items if x.at >= p and through[x.at] and x.budget >= cap:
        ans[i] max= best[x.at, x.budget % pc] + x.budget / pc * pw

print(ans, sep: "\n")

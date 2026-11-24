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
    let (c, w) = item[v]
    for j in c..=cap: small[v, j] max= small[v, j - c] + w
    for k in next[v]:
        for j in 0..=cap: small[k, j] max= small[v, j]
var ans = [small[x.at, min(x.budget, cap)] for x in queries]

for p in V:
    let (pc, pw) = item[p]
    var through: [V]Bool = false
    through[V.first!] = true
    var best: [V, 0..pc]Int = 0
    for v in V if through[v]:
        let (c, w) = item[v]
        if w * pc <= pw * c:
            // Each of the gcd(c, pc) residue cycles walked twice relaxes every residue from every other.
            let g = gcd(c, pc)
            for start in 0..g:
                for step in 0..=2 * (pc / g):
                    let j = (start + step * c) % pc
                    best[v, (j + c) % pc] max= best[v, j] + w - (j + c) / pc * pw
        for k in next[v] if not (v < p < k):
            through[k] = true
            for j in 0..pc: best[k, j] max= best[v, j]
    for (i, x) in queries.items if x.at >= p and through[x.at] and x.budget >= cap:
        ans[i] max= best[x.at, x.budget % pc] + x.budget / pc * pw

print(ans, sep: "\n")

// Source: Codeforces 2115E (Mayflower), https://codeforces.com/contest/2115/problem/E
// limits: 3000 ms, 1024 MB
// Problem: a DAG on n <= 200 vertices with m <= 2000 edges u -> v, u < v, every vertex
//   reachable from 1. Vertex i sells any number of cards of cost c_i <= 200 and power
//   w_i <= 1e9. For q <= 2e5 queries (p, r <= 1e9): walking from 1 to p with r coins, buying
//   on the way, print the most total power.
// Solution: on a fixed path let p be the trader with the best ratio w / c. Any other purchase
//   of at least c_p cards contains a sub-multiset of cost divisible by c_p (prefix sums mod
//   c_p), and swapping it for p's cards loses nothing, so other purchases cost < C^2. Budgets
//   below C^2: an unbounded knapsack carried along the DAG. Larger budgets: for each pivot p,
//   only paths through p and traders no better than p; best[v, j mod c_p] is their power minus
//   w_p per c_p coins, relaxed around the residue cycles. O((n + m) C^2 + nq) total.
let (n, m): (1..=200, 1..=2000) = read()
domain V = 1..=n
let item: [V](cost: 1..=200, worth: 1..=1e9) = read()
let arcs: [0..m](u: V, v: V) = read()
let next = Graph.directed(arcs)
let q: 1..=200_000 = read()
let queries: [0..q](at: V, budget: 1..=1e9) = read()

let cap = V.map(v => item[v].cost).max()! ** 2
var small: [V, 0..=cap]Int = 0
for v in V:
    let (c, w) = item[v]
    for j in 0..=cap - c:
        small[v, j + c] max= small[v, j] + w
    for k in next[v]:
        for j in 0..=cap:
            small[k, j] max= small[v, j]
var ans = [small[x.at, min(x.budget, cap)] for x in queries]

for p in V:
    let (pc, pw) = item[p]
    var reach: [V]Bool = false
    var best: [V, 0..pc]Int = 0
    reach[V.first!] = true
    for v in V if reach[v]:
        let (c, w) = item[v]
        // Unbounded copies of (c, w) on the residue cycle: walking each of the
        // gcd(c, pc) cycles twice lets every residue relax from every other.
        if w * pc <= pw * c:
            let g = gcd(c, pc)
            for start in 0..g:
                for step in 0..=2 * (pc / g):
                    let j = (start + step * c) % pc
                    best[v, (j + c) % pc] max= best[v, j] + w - (j + c) / pc * pw
        for k in next[v] if not (v < p < k):
            reach[k] = true
            for j in 0..pc:
                best[k, j] max= best[v, j]
    for (i, x) in queries.items if x.at >= p and reach[x.at] and x.budget >= cap:
        ans[i] max= best[x.at, x.budget % pc] + x.budget / pc * pw

print(ans, sep: "\n")

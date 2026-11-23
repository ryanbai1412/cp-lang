// Fixed-cost moves over states 0..n: move i takes s to (a_i * s + b_i) mod n
// for cost c_i. Prints the sum of the cheapest costs from state 0.
use std.graph.{fixed_cost_paths}
let (n, k): (2..=2e6, 1..=8) = read()
domain S = 0..n
domain M = 0..k
let moves: [M](a: 0..n, b: 0..n, c: 1..=1e6) = read()
fn step(i: M, s: S) -> ?S = S((moves[i].a * s.ord + moves[i].b) % n)
let dist = fixed_cost_paths(from: [S(0)], moves: List(M), cost: i => moves[i].c, step: step)
var total = 0
for s in S:
    if let d = dist[s]: total += d
print(total)

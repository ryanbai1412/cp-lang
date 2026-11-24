// Source: SAC '22 Code Challenge 2 P3 (Rating Choices), https://dmoj.ca/problem/sac22cc2p3
// Problem: N <= 17 contests and rating R <= 1e9. A directed binary tree: 2^N - 1 lines u v w D
//   say that at node u you take the contest (go to v, gain D, |D| <= 1e4) or skip it (go to
//   w). Starting at node 1, print the maximum rating after the last choice.
// Solution: tree DP. BFS from node 1 gives an order with parents first; in reverse,
//   best[u] = max(D + best[v], best[w]), leaves 0. Print R + best[1]: O(2^N).
// original: corpus/submissions/4121049.cpp
// Bounds: k: 1..=19; r: 0..=1_000_000_000

let k: 1..=19 = read()
let r: 0..=1_000_000_000 = read()
var n = 1
for _ in 0..k:
    n *= 2

// each of the n - 1 lines is a fork at u: take a for w, or take b for free
var to: List<Int> = []
var gain: List<Int> = []
var out: [0..2 * n]List<Int> = []
for _ in 0..n - 1:
    let u: 1..=1_048_576 = read()
    let a: 1..=1_048_576 = read()
    let b: 1..=1_048_576 = read()
    let w: -1_000_000_000..=1_000_000_000 = read()
    out[u - 1].push(to.len)
    to.push(a - 1)
    gain.push(w)
    out[u - 1].push(to.len)
    to.push(b - 1)
    gain.push(0)

var order: List<Int> = []
order.push(0)
var head = 0
while head < order.len:
    let u = order[head]
    head += 1
    for j in 0..out[u].len:
        order.push(to[out[u][j]])

var best: [0..2 * n]Int = 0
for i in 0..order.len:
    let u = order[order.len - 1 - i]
    for j in 0..out[u].len:
        let e = out[u][j]
        let cand = gain[e] + best[to[e]]
        if j == 0 or cand > best[u]:
            best[u] = cand
print(r + best[0])

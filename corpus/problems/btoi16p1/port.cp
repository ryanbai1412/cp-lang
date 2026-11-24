// Source: BOI 2016 Day 1 P1 (Bosses), https://dmoj.ca/problem/btoi16p1
// limits: 1500 ms, 256 MB
// Problem: n <= 5000 employees, each listing the bosses they accept (sum of lists <= 1e4). Build
//   a rooted tree where every boss is acceptable, pay each a positive salary larger than the sum
//   of their direct reports' salaries, and print the least total pay (a solution exists).
// Solution: the best pay is 1 for a leaf and 1 + reports for a boss, so the total is the sum of
//   depths with the root at depth 1. For a fixed root, BFS over boss -> report arcs gives every
//   employee their least depth; skip roots that do not reach everyone. O(n (n + sum k)).
// original: corpus/submissions/5323961.cpp

let n: Int = read()
domain P = 1..=n
var reports: [P]List<(to: P)> = [P] _ => []
for p in P:
    let k: Int = read()
    for _ in 0..k:
        let boss: P = read()
        reports[boss].push((to: p))

var best: ?Int = None
for root in P:
    let depth = bfs(u => reports[u], [root])
    var pay = n
    for p in P:
        let d = depth[p] else:
            pay = -1
            break
        pay += d
    if pay >= 0: best = min(best ?? pay, pay)
print(best!)

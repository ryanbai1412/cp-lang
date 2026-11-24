// Source: BOI 2016 Day 1 P1 (Bosses), https://dmoj.ca/problem/btoi16p1
// limits: 1500 ms, 256 MB
// Problem: n <= 5000 employees, each listing the bosses they accept (sum of lists <= 1e4). Build
//   a rooted tree where every boss is acceptable, pay each a positive salary larger than the sum
//   of their direct reports' salaries, and print the least total pay (a solution exists).
// Solution: the best pay is 1 for a leaf and 1 + reports for a boss, so the total is the sum of
//   depths with the root at depth 1. For a fixed root, BFS over boss -> report arcs gives every
//   employee their least depth; skip roots that do not reach everyone. O(n (n + sum k)).
// original: corpus/submissions/5323961.cpp
// Bounds: n: 2..=5000; accepts: 1..=n
let n: 2..=5000 = read()
domain P = 1..=n
let accepts: [P]List<P> = read()

let reports = Graph.of([(from: boss, arc: p) for p in P for boss in accepts[p]])

// Rooted at `root`, everyone's best pay is their depth counting the root as 1.
fn payroll(root: P) -> ?Int:
    let depth = bfs(reports, [root])
    if P.all(p => depth[p] != None): Some(n + sum(depth[p]! for p in P))
    else: None

print(min(pay for root in P if let pay = payroll(root))!)

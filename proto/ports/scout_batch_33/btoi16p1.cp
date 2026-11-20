// BOI 2016 P1, Bosses: each of n employees lists the people they accept
// as a direct boss. Build a tree with one root where everyone's boss is
// acceptable; a leaf earns 1 and a boss earns 1 more than their reports
// together. Print the least possible total pay.
//
// Total pay is the sum over employees of their depth (root at depth 1),
// so for each choice of root the best tree is the BFS tree over "boss ->
// report" arcs; a root that cannot reach everyone is skipped.
// original: corpus/submissions/5323961.cpp
use std.graph.{bfs}

let n: Int = read()
domain P = 1..=n
var reports: [P]List<(to: P)> = [P] _ => []
for p in P:
    let k: Int = read()
    for _ in 0..k:
        let boss: P = read()
        reports[boss].push((to: p))

var best: ?Int = none
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

// NOI 2005 CongCong and KeKe: a cat chases a uniformly random-walking mouse on
// an unweighted graph, taking up to two greedy shortest-path steps per turn
// (tie-breaking by smallest vertex id). Compute the expected turns until catch.
// original: corpus/submissions/5032733.cpp
use std.graph.{Graph}

let n: 1..=1000 = read()
let e: 1..=1000 = read()
domain Node = 1..=n
let start_cat: Node = read()
let start_mouse: Node = read()
let edges: [e](u: Node, v: Node) = read()
let g = Graph<Node>.undirected(edges)

// All-pairs shortest distances by BFS from each target, plus the cat's unique
// greedy first step from `cat` toward `mouse`.
var dist: [Node, Node]Int = n + 1
for target in Node:
    dist[target, target] = 0
    var q: List<Node> = [target]
    var head = 0
    while head < q.len:
        let u = q[head]
        head += 1
        for v in g.adj(u):
            if dist[target, v] > dist[target, u] + 1:
                dist[target, v] = dist[target, u] + 1
                q.push(v)

var step: [Node, Node]Node = Node.first
for cat in Node:
    for mouse in Node if cat != mouse:                            // WISH P9
        step[cat, mouse] = min_by(g.adj(cat), v => (dist[mouse, v], v)).item   // WISH P13

// Expected turns from (cat, mouse): the cat moves up to twice deterministically,
// then the mouse stays or moves uniformly to one of its neighbors.
@memo                                                             // WISH P4
fn expected(cat: Node, mouse: Node) -> Float:                     // WISH P4
    if cat == mouse: return 0.0
    let c1 = step[cat, mouse]
    if c1 == mouse: return 1.0
    let c2 = step[c1, mouse]
    if c2 == mouse: return 1.0
    let deg = g.adj(mouse).len
    var total = expected(c2, mouse)
    for nxt in g.adj(mouse): total += expected(c2, nxt)
    1.0 + total / Float(deg + 1)                                  // WISH P4

print(expected(start_cat, start_mouse), prec: 3)                  // WISH P4

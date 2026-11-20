// DMOPC '20 C7 P5: n items with costs a[0..n], and m pairs (u, v, w) paying
// reward w when both items u and v are chosen. Maximize total reward minus
// total cost.
//
// Classic project-selection min-cut: take every reward up front, then pay
// either to forfeit a pair (cut src -> pair) or to buy an item (cut item ->
// snk), with infinite arcs tying each pair to its two items. Replaces the
// original's 5000 x 5000 dense matrix and manual capacity-scaling DFS with
// `std.flow.FlowGraph` (sparse Dinic).
// original: corpus/submissions/4370353.cpp
use std.flow.{FlowGraph}

let (n, m): (Int, Int) = read()
domain Item = 0..n
domain Pair = 0..m

let cost: [Item]Int = read()
// Flatten `Item + Pair + Src + Snk` into one domain for `FlowGraph`.
domain Node = 0..n + m + 2
let (src, snk) = (Node(n + m), Node(n + m + 1))
const INF = 1000000000000000000

var net = FlowGraph<Node>.new()
for i in Item:
    net.add(Node(i.to_idx), snk, cost[i])

var total_reward = 0
for e in Pair:
    let (u, v, w): (Int, Int, Int) = read()
    let pnode = Node(n + e.to_idx)
    total_reward += w
    net.add(src, pnode, w)
    net.add(pnode, Node(u - 1), INF)
    net.add(pnode, Node(v - 1), INF)

print(total_reward - net.max_flow(src, snk))

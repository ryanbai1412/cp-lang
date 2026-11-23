// Source: NOI 2005 P4 (CongCong and KoKo), https://dmoj.ca/problem/noi05p4
// Problem: an undirected graph, N, E <= 1000, cat at C and mouse at M connected. Each turn the cat
//   takes up to two steps, each to the neighbour nearest the mouse (smallest label on ties), then
//   the mouse stays or moves to a neighbour, all choices equally likely. Print the expected
//   number of turns until they meet, to 3 decimals.
// Solution: the cat's moves are deterministic and the distance drops every turn, so the states
//   (cat, mouse) form a DAG. BFS from every vertex gives all distances and step[cat, mouse]; a
//   memoised E(cat, mouse) = 1 + the average of E over the mouse's deg + 1 choices after the
//   cat's two steps, or 1 if a step catches it. O(n (n + e)).
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
        for v in g.out(u):
            if dist[target, v] > dist[target, u] + 1:
                dist[target, v] = dist[target, u] + 1
                q.push(v)

var step: [Node, Node]Node = Node.first!
for cat in Node:
    for mouse in Node if cat != mouse:                                        // WISH P9
        step[cat, mouse] = g.out(cat).min(by: v => (dist[mouse, v], v))!  // WISH P13

// Expected turns from (cat, mouse): the cat moves up to twice deterministically,
// then the mouse stays or moves uniformly to one of its neighbors.
@memo                                        // WISH P4
fn expected(cat: Node, mouse: Node) -> F64:  // WISH P4
    if cat == mouse: return 0.0
    let c1 = step[cat, mouse]
    if c1 == mouse: return 1.0
    let c2 = step[c1, mouse]
    if c2 == mouse: return 1.0
    let deg = g[mouse].len
    var total = expected(c2, mouse)
    for nxt in g.out(mouse): total += expected(c2, nxt)
    1.0 + total / F64(deg + 1)  // WISH P4

print(expected(start_cat, start_mouse), precision: 3)  // WISH P4

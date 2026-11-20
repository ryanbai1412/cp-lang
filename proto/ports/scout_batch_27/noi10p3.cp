// NOI 2010 Altitude: an n x n grid of city blocks, so (n + 1) x (n + 1)
// crossings. Every street segment carries a number of walkers in each
// direction. The north-west corner has height 0, the south-east corner
// height 1, and walking up costs the climb times the walkers. Choose the
// other heights to minimise the total cost and print it.
//
// Some best answer uses heights 0 and 1 only, with the low crossings in one
// piece around the north-west corner. Its border is a path through the
// blocks from the south-west outside to the north-east outside, keeping the
// low side on its left; stepping over a segment pays the walkers going from
// the low end to the high end. So it is one shortest path over the blocks.
//
// Input: n, then the walkers west to east ((n + 1) rows of n), north to
// south (n rows of n + 1), east to west and south to north.
// original: corpus/submissions/5672180.cpp
use std.graph.{dijkstra}

let n: Int = read()
domain X = 0..=n                    // crossings along one axis
domain B = 0..n                     // blocks along one axis
domain Node = 0..n * n + 1          // the blocks, then the north-east outside
let out = Node(n * n)
fn block(i: Int, j: Int) -> Node = Node(i * n + j)

let east: [X, B]Int = read()        // segment right of crossing (i, j)
let south: [B, X]Int = read()       // segment below crossing (i, j)
let west: [X, B]Int = read()
let north: [B, X]Int = read()

// Leaving block (i, j): up over its top side, right over its right side
// (low end first for both), or down and left the other way round.
fn steps(u: Node) -> List<(to: Node, cost: Int)>:
    if u == out: return []
    let (i, j) = (B(u.to_idx / n), B(u.to_idx % n))
    let (top, bottom, left, right) = (X(i.to_idx), X(i.to_idx + 1), X(j.to_idx), X(j.to_idx + 1))
    var to: List<(to: Node, cost: Int)> = []
    to.push((to: if i.to_idx == 0: out else: block(i.to_idx - 1, j.to_idx), cost: east[top, j]))
    to.push((to: if j.to_idx == n - 1: out else: block(i.to_idx, j.to_idx + 1), cost: south[i, right]))
    if i.to_idx < n - 1: to.push((to: block(i.to_idx + 1, j.to_idx), cost: west[bottom, j]))
    if j.to_idx > 0: to.push((to: block(i.to_idx, j.to_idx - 1), cost: north[i, left]))
    to

// Entering from the south-west outside: over the west side or the south side.
var starts: List<(Node, Int)> = []
for i in B:
    starts.push((block(i.to_idx, 0), south[i, X(0)]))
    starts.push((block(n - 1, i.to_idx), east[X(n), i]))

print(dijkstra(steps, starts, s => s.cost)[out]!)

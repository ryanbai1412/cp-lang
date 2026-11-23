// Source: NOI 2007 P6, https://dmoj.ca/problem/noi07p6
// Problem: a tree on N <= 1000 cities hides a thief who moves arbitrarily fast; a detective in
//   his city or on his road catches him. Steps: L x lands a detective at x, B x recalls one,
//   M x y moves one along an edge. Print S, the fewest detectives that surely catch him, the
//   step count T <= 20000 and the plan; partial score when S is above the optimum.
// Solution: not optimal, it uses S = leaves - 1. Root the tree at its first leaf, land a
//   detective on every other leaf, then sweep deepest first with a heap on (depth, vertex):
//   each vertex, once everything below it is clear, sends one detective to its parent and the
//   others stay behind as guards. O(n log n).
// original: corpus/submissions/3756979.cpp
let n: 3..=1000 = read()
domain Node = 1..=n

var adj: [Node]List<Node> = []
for _ in 0..n - 1:
    let u: Node = read()
    let v: Node = read()
    adj[u].push(v)
    adj[v].push(u)

var leaves: List<Node> = []
for u in Node:
    if adj[u].len == 1:
        leaves.push(u)

let root = leaves[0]
let tree = Rooted(adj, root)

var pq = MinHeap<(Int, Int)>()
for i in 1..leaves.len:
    let u = leaves[i]
    pq.push((-tree.depth(u), -u.idx))

var vis: [Node]Bool = false
var moves: List<(Node, Node)> = []
while let top = pq.pop():
    let u = Node(-top.1 + 1)
    if vis[u]: continue
    vis[u] = true
    if let p = tree.parent(u):
        moves.push((u, p))
        pq.push((-tree.depth(p), -p.idx))

print(leaves.len - 1)
print(leaves.len - 1 + moves.len)
for i in 1..leaves.len:
    print("L", leaves[i])
for mv in moves:
    print("M", mv.0, mv.1)

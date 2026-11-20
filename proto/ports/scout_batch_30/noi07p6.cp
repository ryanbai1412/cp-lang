// NOI 2007 P6: given a tree on n >= 3 vertices, place pebbles on all leaves
// except one chosen root leaf and pull them deepest-first along parent edges
// until every vertex has been merged into the root.
// Root the tree at its first leaf via Rooted(adj, root), then run a deepest-
// first greedy walk using a negated MinHeap over (depth, vertex).
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

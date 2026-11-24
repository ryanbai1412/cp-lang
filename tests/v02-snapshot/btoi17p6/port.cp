// Source: BOI 2017 Day 2 (Cat in a Tree), https://dmoj.ca/problem/btoi17p6
// Problem: a tree on n <= 2e5 nodes rooted at 0, given as the parents x_i < i, and d <= 2e5.
//   Print the most nodes that can be marked so every two marked nodes are >= d edges apart.
// Solution: greedy bottom-up: a subtree keeps (count, depth of its shallowest mark), a leaf
//   marks itself. Labels only grow downward, so a reverse label scan joins each child into
//   its parent: if the two shallowest marks are >= d apart keep both, else drop the
//   shallower one (one fewer mark, the other is nearest). O(n).
let n: 1..=2e5 = read()
let d: 1..=2e5 = read()
domain V = 0..n
domain Kid = 1..n

fn random_parents() -> [Kid]V:
    [Kid] k => V(rand(0, k.label - 1))

let parent: [Kid]V = read(gen: random_parents)

// marks chosen inside a subtree, and the depth of the shallowest one
type Pick = (count: Int, near: Int)

// Two picks that hang off the same node: keep both if their shallowest marks
// are far enough apart, otherwise give up the shallower mark.
fn join(a: Pick, b: Pick) -> Pick:
    if a.near + b.near >= d: (a.count + b.count, min(a.near, b.near))
    else: (a.count + b.count - 1, max(a.near, b.near))

var pick: [V]Pick = (1, 0)  // on its own, a node marks itself
for k in Kid.rev:           // children are finished before parents
    let below = pick[V(k.label)]
    pick[parent[k]] = join(pick[parent[k]], (below.count, below.near + 1))
print(pick[V.first!].count)

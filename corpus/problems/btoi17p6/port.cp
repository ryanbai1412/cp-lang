// Source: BOI 2017 Day 2 (Cat in a Tree), https://dmoj.ca/problem/btoi17p6
// limits: 1000 ms, 1024 MB
// Problem: a tree on n <= 2e5 nodes rooted at 0, given as the parents x_i < i, and d <= 2e5.
//   Print the most nodes that can be marked so every two marked nodes are >= d edges apart.
// Solution: greedy bottom-up: a subtree keeps (count, depth of its shallowest mark), a leaf
//   marks itself. `tree.up` joins each child into
//   its parent: if the two shallowest marks are >= d apart keep both, else drop the
//   shallower one (one fewer mark, the other is nearest). O(n).
// Bounds: n, d: 1..=2e5
let n: 1..=2e5 = read()
let d: 1..=2e5 = read()
domain V = 0..n
domain Kid = 1..n

fn random_parents() -> [Kid]V:
    [Kid] k => V(rand(0, k.label - 1))

let parent: [Kid]V = read(gen: random_parents)
let tree = Rooted.from_parents([V] v => Kid.at(v.ord).map(k => parent[k]))

// marks chosen inside a subtree, and the depth of the shallowest one
type Pick = (count: Int, near: Int)

// Two picks that hang off the same node: keep both if their shallowest marks
// are far enough apart, otherwise give up the shallower mark.
fn join(a: Pick, b: Pick) -> Pick:
    if a.near + b.near >= d: (count: a.count + b.count, near: min(a.near, b.near))
    else: (count: a.count + b.count - 1, near: max(a.near, b.near))

let best = tree.up(
    Monoid<Pick>(id: (count: 0, near: 2 * d), op: join),
    lift: (p, _, _) => (count: p.count, near: p.near + 1),
    finish: (kids, _) => join(kids, (count: 1, near: 0)),
)
print(best[V.first!].count)

// Source: AtCoder ABC479 E (Cut and Add), https://atcoder.jp/contests/abc479/tasks/abc479_e
// Problem: a tree with an odd number N <= 2e5 of vertices, edges numbered 1..N-1. For each edge
//   i, removing it splits the tree; add i to x_v for every v in the larger part. Print x_1..x_N.
// Solution: root at 1. The edge above c (`lower`) adds i to sub(c) when size(c) > N - size(c),
//   else to everything outside sub(c): add i to a base and -i at c. Pushing the offsets down in
//   preorder gives every x in O(N).

let n: 3..=2e5 = read()
domain V = 1..=n
domain E = 1..n
let edges: [E](u: V, v: V) = read()
let tree = Rooted(Graph.undirected(edges), V.first!)

// cutting edge e above c adds e to sub(c) if that is the larger side, else to everything but
// sub(c): x[v] is base plus the `down` of v and its ancestors
var base = 0
var down: [V]Int = 0
for (e, (u, v)) in edges.items:
    let c = tree.lower(u, v)
    if tree.size(c) > n - tree.size(c):
        down[c] += e.label
    else:
        base += e.label
        down[c] -= e.label
var x: [V]Int = 0
for v in tree.order:
    x[v] = down[v] + (if let p = tree.parent(v): x[p] else: base)
print(x)

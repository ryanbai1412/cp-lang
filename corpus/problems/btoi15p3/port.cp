// Source: Baltic OI 2015 (Network), https://dmoj.ca/problem/btoi15p3
// limits: 1000 ms, 256 MB
// Problem: a tree on 3 <= n <= 5e5 computers. Add the fewest links so that the network stays
//   connected after any one link breaks; print their number and the links (any optimal set).
// Solution: every leaf needs a new link, so ceil(L / 2) for L leaves is a lower bound, and it
//   is reached: root the tree at an inner vertex, list the leaves in preorder and join leaf i
//   to leaf i + L/2 for i < ceil(L / 2). The leaves under any tree edge are a block of that
//   list, shorter than L, so some new link leaves the block and covers the edge. O(n).
// original: corpus/submissions/5640865.cpp
// Bounds: n: 3..=5e5; edges.u, edges.v: 1..=n
let n: 3..=5e5 = read()
domain V = 1..=n
domain E = 0..n - 1
let edges: [E](u: V, v: V) = read()
let g = Graph.undirected(edges)
let tree = Rooted(g, V.find(v => g.deg(v) > 1)!)

let leaves = [v for v in tree.at if g.deg(v) == 1]
let half = (leaves.len + 1) / 2
print(half)
for i in 0..half: print(leaves[i], leaves[i + leaves.len / 2])

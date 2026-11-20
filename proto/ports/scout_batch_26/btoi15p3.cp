// Baltic OI '15 P3 - Network (15pt): add as few links as possible to a tree
// so that it stays connected when any one link fails. Print the number of
// new links and the links (any optimal answer).
// Every leaf needs a new link, so ceil(leaves / 2) is a lower bound. List the
// leaves in preorder from an inner root and join leaf i to leaf i + k/2:
// the two ends of each new link lie on different sides of every tree edge
// that has leaves on both halves, which covers every edge.
// original: corpus/submissions/5640865.cpp
use std.graph.{Graph}
use std.tree.{Rooted}

let n: 3..=5e5 = read()
domain V = 1..=n
domain E = 0..(n - 1)
let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)

var root = V.first
for v in V:
    if g.deg(v) > 1: root = v
let t = Rooted<V>.new(u => g.adj(u), root)

var leaves: List<V> = []
for p in Rooted<V>.Pre:
    if g.deg(t.at[p]) == 1: leaves.push(t.at[p])

let k = leaves.len
print((k + 1) / 2)
for i in 0..(k + 1) / 2: print(leaves[i], leaves[i + k / 2])

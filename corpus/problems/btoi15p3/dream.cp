// BOI 2015 Network: fewest new links making a tree 2-edge-connected. Root at an inner vertex,
// list the leaves in preorder and link each leaf of the first half to the one half a list on.
let n: 3..=5e5 = read()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let t = Rooted(g, V.find(v => g.deg(v) > 1)!)
let leaves = [v for v in t.at if g.deg(v) == 1]
let half = (leaves.len + 1) / 2
print(half)
for i in 0..half: print(leaves[i], leaves[i + leaves.len / 2])

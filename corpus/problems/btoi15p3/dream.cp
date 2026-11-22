// BOI 2015 Network: fewest new links making a tree 2-edge-connected. List the leaves in DFS
// order and link each leaf to the one half the list ahead, wrapping round.
domain V = 1..=read Int
let g = Graph.undirected(read [V.len - 1](u: V, v: V))
let leaves = g.dfs_order(from: V.find(v => g.deg(v) > 1)!).filter(v => g.deg(v) == 1).list()
let half = (leaves.len + 1) / 2
print(half)
for i in leaves.keys.take(half): print(leaves[i], leaves.ring.step(i, half))

// btoi15p3 Network: the fewest new links that make a tree 2-edge-connected. Take the leaves in
// DFS order and link leaf i to leaf i + ceil(L / 2): every tree edge then separates two
// linked leaves, so it lies on a cycle.
let n = read<3..=500_000>()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V)>())
let t = Rooted(g, root: V.find(v => g[v].len > 1)!)
let leaves = t.Pre.map(p => t.at(p)).filter(v => g[v].len == 1).collect()
let half = (leaves.len + 1) / 2
print(half)
for i in 0..half:
    print(leaves[i], leaves[(i + half) % leaves.len])

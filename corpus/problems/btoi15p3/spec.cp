// btoi15p3 Network: the fewest new links that make a tree 2-edge-connected. Take the leaves in
// DFS order and link leaf i to leaf i + ceil(L / 2), wrapping: every tree edge then separates
// two linked leaves, so it lies on a cycle. Any optimal set is accepted (checker.py).
let n = read<3..=500_000>()
domain V = 1..=n
let g = Graph.undirected(read<Tree<V>>())
let t = Rooted(g, root: V.find(v => g[v].len > 1)!)
let leaves = t.at.filter(v => g[v].len == 1).collect()
let half = (leaves.len + 1) / 2
print(half)
for (a, b) in zip(leaves.take(half), leaves.rotated_left(half)):
    print(a, b)

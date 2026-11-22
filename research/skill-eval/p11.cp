let n = read<1..=2e5>()
domain V = 1..=n
let edges = read<Tree<V>>()
let g = Graph.undirected(edges)
let t = Rooted(g, root: V(1))
let root_sum = V.all.map(v => t.depth(v)).sum()
let ans = t.down(root_sum, (cur, v) => if v == V(1): cur else: cur + V.count - 2 * t.size(v))
print(ans)

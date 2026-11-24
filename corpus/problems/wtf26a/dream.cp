// AtCoder WTF 2026 A.
//
// Kruskal over the edges by decreasing weight builds the reconstruction tree:
// leaves are vertices, edge e becomes node `n + e`, and the two sets it joins
// hang under it as `Cut`s. A cut keeps its own size and `w` times the other
// side's size; summed from the root down, `total[v] * size` is v's cost, capped
// at `max_w` (the heaviest edge times n - 1; compared by division so the
// product never overflows). A second Kruskal over those costs, again by
// decreasing weight, adds `cost * leaves(a) * leaves(b)` at each join.
type Z = Mod<998_244_353>

@!multitest
let n: 2..=2e5 = read()
domain U = 1..=n
let edges = read<[0..n - 1](u: U, v: U, w: 1..=1e9)>().sorted(by: e => desc(e.w))
let max_w = edges[0].w * (n - 1)

let tree = KruskalTree<U>(edges)
domain Node = tree.nodes
let total: [Node]Int = tree.down(0, (above, v) => above + tree.weight(v) * tree.other_size(v))
let cost = [v => min(total[v] * tree.size(v), max_w, saturating) for v in tree.non_root]

let joins = KruskalTree<Node>(cost.edges(to: tree.parent).sorted(by: e => desc(e.w)))
print(sum(Z(e.w) * joins.leaves(e.a) * joins.leaves(e.b) for e in joins.merges))

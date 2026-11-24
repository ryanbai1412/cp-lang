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
let edges = read<[0..n - 1](u: U, v: U, w: 1..=1e12)>().sorted(by: e => desc(e.w))
let max_w = edges[0].w * (n - 1)

let (domain Node, kt) = kruskal_tree(edges, w: e => e)   // `edges` is heaviest first
let tree = kt.tree
let leaves = kt.up(+, leaf: _ => 1)
fn cut(v: Node) -> Int:                        // the edge above v times the other side's size
    guard let p = tree.parent(v) else: return 0
    edges[kt.w[p]!].w * (leaves[p] - leaves[v])
let total = tree.down(0, (above, v) => above + cut(v))
let joins = [
    (cost: if total[v] > max_w / leaves[v]: max_w else: total[v] * leaves[v], v:, p:)
    for v in Node if let p = tree.parent(v)
].sorted(by: j => desc(j.cost))

var sets = DsuOf<Node, Int>(v => if v.ord < n: 1 else: 0, (a, b) => a + b)
var ans: Z = 0
for j in joins:
    let (a, b) = (sets.get(j.v), sets.get(j.p))
    if sets.unite(j.v, j.p): ans += Z(a) * b * j.cost
print(ans)

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

struct Cut(w: Int, size: Int, parent: Node)

@!multitest
let n: Int = read()
domain U = 1..=n
domain Node = 0..2 * n - 1
let edges = read<[0..n - 1](u: U, v: U, w: Int)>().sorted(by: e => e.w, desc: true)
let max_w = edges[0].w * (n - 1)

var cut: [Node]?Cut = none
var sets = Dsu<U>()
var top = [U] u => Node(u.ord)
for (e, edge) in edges.items:
    let (su, sv) = (sets.size(edge.u), sets.size(edge.v))
    let (tu, tv) = (top[sets.find(edge.u)], top[sets.find(edge.v)])
    let joined = Node(n + e)
    cut[tu] = some(Cut(edge.w * sv, su, joined))
    cut[tv] = some(Cut(edge.w * su, sv, joined))
    let (kept, _) = sets.merge(edge.u, edge.v)!
    top[kept] = joined

var total: [Node]Int = 0
for v in Node.rev if let c = cut[v]: total[v] = c.w + total[c.parent]

let joins = [
    (
        cost: if total[v] > max_w / c.size: max_w else: total[v] * c.size,
        v:,
        parent: c.parent,
    ) for v in Node if let c = cut[v]
].sorted(by: j => j.cost, desc: true)

var leaves = DsuOf<Node, Int>(v => if v.ord < n: 1 else: 0, (a, b) => a + b)
var ans: Z = 0
for j in joins:
    let (a, b) = (leaves.get(j.v), leaves.get(j.parent))
    if leaves.unite(j.v, j.parent): ans += Z(a) * b * j.cost
print(ans)

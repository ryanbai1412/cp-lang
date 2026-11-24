// AtCoder World Tour Finals 2026 A (Min Cut of Graph of Min Weight), tuned.
// port.cp's two Kruskals, with the joins pushed into a reserved list instead of a filtered
// comprehension, and the second DSU a plain Dsu with a leaf count per root, so each join
// finds its two roots once.

type Z = Mod<998_244_353>

struct Cut(w: Int, size: Int, parent: Node)

@!multitest
let n: Int = read()
domain U = 1..=n
domain Node = 0..2 * n - 1
let edges = read<[0..n - 1](u: U, v: U, w: Int)>().sorted(by: e => desc(e.w))
let max_w = edges[0].w * (n - 1)

var cut: [Node]?Cut = None
var sets = Dsu<U>()
var top = [U] u => Node(u.ord)
for (e, edge) in edges.items:
    let (su, sv) = (sets.size(edge.u), sets.size(edge.v))
    let (tu, tv) = (top[sets.find(edge.u)], top[sets.find(edge.v)])
    let joined = Node(n + e)
    cut[tu] = Some(Cut(edge.w * sv, su, joined))
    cut[tv] = Some(Cut(edge.w * su, sv, joined))
    let (kept, _) = sets.merge(edge.u, edge.v)!
    top[kept] = joined

var total: [Node]Int = 0
for v in Node.rev if let c = cut[v]: total[v] = c.w + total[c.parent]

var joins: List<(cost: Int, v: Node, parent: Node)> = []
joins.reserve(2 * n)
for v in Node if let c = cut[v]:
    joins.push(
        (cost: if total[v] > max_w / c.size: max_w else: total[v] * c.size, v:, parent: c.parent)
    )
joins.sort(by: j => desc(j.cost))

var leaves = Dsu<Node>()
var freq = [Node] v => if v.ord < n: 1 else: 0
var ans: Z = 0
for j in joins:
    let (a, b) = (leaves.find(j.v), leaves.find(j.parent))
    ans += Z(freq[a]) * freq[b] * j.cost
    let (kept, _) = leaves.merge(a, b)!
    freq[kept] = freq[a] + freq[b]
print(ans)

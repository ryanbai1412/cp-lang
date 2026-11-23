// Source: AtCoder World Tour Finals 2026 A (Min Cut of Graph of Min Weight),
//   https://atcoder.jp/contests/awtf2026algo/tasks/awtf2026algo_a
// Problem: a tree of N <= 2.5e5 vertices, weights C <= 1e12. G is the complete graph whose edge
//   (i, j) has capacity min weight on the tree path i-j; print the sum over i < j of the min cut
//   f(i, j) in G, mod 998244353. Multitest, S <= 1.25e5 cases, sum N <= 2.5e5.
// Solution: Kruskal by decreasing weight builds the reconstruction tree; edge (i, j) of G has
//   the weight at their join, so cutting a node's subtree costs size * the sum over its
//   ancestor joins of w * (other side's size). These cuts suffice, so f(i, j) is the least cut
//   on the tree path i-j: a second Kruskal over the cuts by decreasing cost adds
//   cost * leaves * leaves per join. A leaf's cut is at most max weight * (n - 1), so capping
//   costs there keeps every path minimum and avoids overflow. Two sorts and DSUs, O(N log N).

type Z = Mod<998_244_353>

struct Cut(w: Int, size: Int, parent: Node)

@!multitest
let n: Int = read()
domain U = 1..=n
domain Node = 0..2 * n - 1
let edges = read<[0..n - 1](u: U, v: U, w: Int)>().sorted(by: e => e.w, desc: true)
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

// Source: Codeforces 2152H2 (Victorious Coloring, Hard Version),
//   https://codeforces.com/contest/2152/problem/H2
// limits: 3000 ms, 1024 MB
// Problem: a tree, n <= 2.5e5, edge weights <= 1e9. For vertex weights x >= 0 let f(x) be the min
//   over nonempty vertex sets S of x(S) + w(edges leaving S). For each of q <= 2.5e5 queries
//   l <= 1e9, print the least sum of x with f(x) >= l. Multitest, sums of n and q <= 2.5e5.
// Solution: for a vertex set C, the least x(C) with every S in C paying >= l is convex piecewise
//   linear, F_C(l) = sum of max(0, l - b) over a multiset B_C of breakpoints; B of one vertex is
//   {its weighted degree}. Join edges heaviest first (DSU): melding two components adds their
//   B's (skew heaps), and the whole component with boundary D lifts F to max(F, l - D); when
//   D < min B, pop breakpoints while F lies below that line and push the crossing ones. The
//   answer is l * k - (sum of the k breakpoints < l): O((n + q) log n).

// Multisets of breakpoints as skew heaps in one arena of nodes; a set is its ?root.
struct Breaks:
    node: List<(at: Int, cnt: Int, l: ?Int, r: ?Int)>

    fn single(mut self, at: Int, cnt: Int) -> ?Int:
        node.push((at:, cnt:, l: None, r: None))
        node.len - 1

    fn meld(mut self, oa: ?Int, ob: ?Int) -> ?Int:
        var a = oa ?? return ob
        var b = ob ?? return oa
        if node[a].at > node[b].at: (a, b) = (b, a)
        (node[a].l, node[a].r) = (self.meld(node[a].r, b), node[a].l)
        a

    fn rest(mut self, h: Int) -> ?Int = self.meld(node[h].l, node[h].r)

@!multitest
let n: 2..=250_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V, w: 1..=1e9) = read()
var bound: [V]Int = 0
for (u, v, w) in edges:
    bound[u] += w
    bound[v] += w

var br = Breaks([])
var set: [V]?Int = [V] v => br.single(bound[v], 1)
var dsu = Dsu<V>()

// Raise F, the multiset rooted at `h`, to max(F, l - d) at the integers.
fn raise(h: Int, d: Int) -> ?Int:
    var rest: ?Int = h
    var (pos, slope, val) = (br.node[h].at, 0, 0)  // F(pos) = val, slope just right of pos
    while let b = rest and val + slope * (br.node[b].at - pos) < br.node[b].at - d:
        let at = br.node[b].at
        (val, pos, slope) = (val + slope * (at - pos), at, slope + br.node[b].cnt)
        rest = br.rest(b)
    // slope >= 2 here: the root pops (d < its at), F stays below l - d while slope is 1, and
    // with every breakpoint popped slope is the component's vertex count, at least 2.
    // l - d meets F at pos + (pos - val - d) / (slope - 1): split the jump of slope - 1 there.
    let (step, part) = (pos - val - d).divmod(slope - 1)
    let cross = pos + step
    if part > 0: rest = br.meld(br.single(cross + 1, part), rest)
    rest = br.meld(br.single(cross, slope - 1 - part), rest)
    br.meld(br.single(d, 1), rest)

for (u, v, w) in edges.sorted(by: e => desc(e.w)):
    let (kept, merged) = dsu.merge(u, v)!
    set[kept] = br.meld(set[kept], set[merged])
    bound[kept] += bound[merged] - 2 * w
    let top = set[kept]!  // never empty: each vertex brings one breakpoint
    if bound[kept] < br.node[top].at: set[kept] = raise(top, bound[kept])

var breaks: List<Int> = []
var h = set[dsu.find(V.first!)]
while let b = h:
    repeat br.node[b].cnt: breaks.push(br.node[b].at)
    h = br.rest(b)

let below = prefix(breaks)
let q: 1..=250_000 = read()
let asks: [0..q]1..=1e9 = read()
for l in asks:
    let k = breaks.partition_point(b => b >= l)
    print(l * k - below.cut(k))

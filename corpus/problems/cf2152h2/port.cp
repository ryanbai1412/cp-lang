// Source: Codeforces 2152H2 (Victorious Coloring, Hard Version),
//   https://codeforces.com/contest/2152/problem/H2
// Problem: a tree, n <= 2.5e5, edge weights <= 1e9. For vertex weights x >= 0 let f(x) be the min
//   over nonempty vertex sets S of x(S) + w(edges leaving S). For each of q <= 2.5e5 queries
//   l <= 1e9, print the least sum of x with f(x) >= l. Multitest, sums of n and q <= 2.5e5.
// Solution: for a vertex set C, the least x(C) with every S in C paying >= l is convex piecewise
//   linear, F_C(l) = sum of max(0, l - b) over a multiset B_C of breakpoints; B of one vertex is
//   {its weighted degree}. Join edges heaviest first (DSU): melding two components adds their
//   B's (skew heaps), and the whole component with boundary D lifts F to max(F, l - D); when
//   D < min B, pop breakpoints while F lies below that line and push the crossing ones. The
//   answer is l * k - (sum of the k breakpoints < l): O((n + q) log n).

// Multisets of breakpoints as skew heaps in one arena; a set is its ?root.
struct Breaks:
    at: List<Int>
    cnt: List<Int>
    kids: List<(l: ?Int, r: ?Int)>

    fn single(mut self, b: Int, c: Int) -> ?Int:
        at.push(b)
        cnt.push(c)
        kids.push((l: None, r: None))
        at.len - 1

    fn meld(mut self, oa: ?Int, ob: ?Int) -> ?Int:
        var a = oa ?? return ob
        var b = ob ?? return oa
        if at[a] > at[b]: (a, b) = (b, a)
        kids[a] = (l: self.meld(kids[a].r, b), r: kids[a].l)
        a

    fn rest(mut self, h: Int) -> ?Int = self.meld(kids[h].l, kids[h].r)

@!multitest
let n: 2..=250_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V, w: 1..=1e9) = read()
var bound: [V]Int = 0
for (u, v, w) in edges:
    bound[u] += w
    bound[v] += w

var br = Breaks([], [], [])
var set: [V]?Int = [V] v => br.single(bound[v], 1)
var dsu = Dsu<V>()

// Raise the breakpoints with smallest `h` (F) to max(F, l - d) at the integers.
fn raise(h: Int, d: Int) -> ?Int:
    var rest: ?Int = h
    var (pos, slope, val) = (br.at[h], 0, 0)  // F(pos) = val, slope just right of pos
    while let b = rest and val + slope * (br.at[b] - pos) < br.at[b] - d:
        (val, pos, slope) = (val + slope * (br.at[b] - pos), br.at[b], slope + br.cnt[b])
        rest = br.rest(b)
    // l - d meets F at pos + (pos - val - d) / (slope - 1): split the jump of slope - 1 there.
    let (step, part) = (pos - val - d).divmod(slope - 1)
    let cross = pos + step
    if part > 0: rest = br.meld(br.single(cross + 1, part), rest)
    rest = br.meld(br.single(cross, slope - 1 - part), rest)
    br.meld(br.single(d, 1), rest)

for (u, v, w) in edges.sorted(by: e => e.w, desc: true):
    let (kept, merged) = dsu.merge(u, v)!
    set[kept] = br.meld(set[kept], set[merged])
    bound[kept] += bound[merged] - 2 * w
    let top = set[kept]!  // never empty: each vertex brings one breakpoint
    if bound[kept] < br.at[top]: set[kept] = raise(top, bound[kept])

var breaks: List<Int> = []
var h = set[dsu.find(V.first!)]
while let b = h:
    repeat br.cnt[b]: breaks.push(br.at[b])
    h = br.rest(b)

let below = prefix(breaks)
let q: 1..=250_000 = read()
let asks: [0..q]1..=1e9 = read()
for l in asks:
    let k = breaks.partition_point(b => b >= l)
    print(l * Int(k) - below.prefix(k))

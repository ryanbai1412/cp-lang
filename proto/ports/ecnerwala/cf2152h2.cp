// Codeforces 2152 H2: Victorious Coloring (Hard Version).
// For a set C of vertices let F_C(l) be the least x(C) such that every
// nonempty S within C pays x(S) + w(boundary of S) >= l. F_C is convex and
// piecewise linear with integer breakpoints: F_C(l) = sum over its multiset
// B_C of max(0, l - b).
// Join the tree's edges heaviest first, keeping the components. A vertex v
// alone needs x(v) >= l - deg(v), so B = {deg v}. Joining A and A' keeps the
// two sides' constraints independent (B_A plus B_A'), and adds one: the
// whole component, with boundary D, needs x >= l - D. That raises F to
// max(F, l - D) at the integers, which only matters when D < min B: walk
// the breakpoints while F(b) < b - D, then re-enter F where the line l - D
// meets it, splitting the slope jump across the two integers around the
// crossing. The answer for l is F_V(l).

// Multisets of breakpoints as skew heaps in one arena; a set is its ?root.
struct Breaks:
    at: List<Int>
    cnt: List<Int>
    kids: List<(l: ?Int, r: ?Int)>

    fn single(mut self, b: Int, c: Int) -> ?Int:
        at.push(b)
        cnt.push(c)
        kids.push((l: none, r: none))
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

// Raise the breakpoints `h` (F) to max(F, l - d) at the integers.
fn raise(h: ?Int, d: Int) -> ?Int:
    var rest = h
    var (pos, slope, val) = (br.at[h!], 0, 0)  // F(pos) = val, slope just right of pos
    while let b = rest and val + slope * (br.at[b] - pos) < br.at[b] - d:
        val += slope * (br.at[b] - pos)
        pos = br.at[b]
        slope += br.cnt[b]
        rest = br.rest(b)
    let (step, part) = ((pos - val - d) / (slope - 1), (pos - val - d) % (slope - 1))
    let cross = pos + step
    if part > 0: rest = br.meld(br.single(cross + 1, part), rest)
    rest = br.meld(br.single(cross, slope - 1 - part), rest)
    br.meld(br.single(d, 1), rest)

for (u, v, w) in edges.sorted(by: e => -e.w):
    let (kept, merged) = dsu.merge(u, v)!
    set[kept] = br.meld(set[kept], set[merged])
    bound[kept] += bound[merged] - 2 * w
    if bound[kept] < br.at[set[kept]!]: set[kept] = raise(set[kept], bound[kept])

var breaks: List<Int> = []
var h = set[dsu.find(V.first)]
while let b = h:
    for _ in 0..br.cnt[b]: breaks.push(br.at[b])
    h = br.rest(b)

let below = prefix(breaks)
let q: 1..=250_000 = read()
let asks: [0..q]1..=1e9 = read()
for l in asks:
    let k = breaks.first_ge(l) ?? breaks.len
    print(l * k - below.prefix(k))

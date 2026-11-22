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
use std.dsu.{Dsu}
use std.heap.{MeldHeap}

@!multitest
let n: 2..=250_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V, w: 1..=1e9) = read()
var bound: [V]Int = 0
for (u, v, w) in edges:
    bound[u] += w
    bound[v] += w
var set: [V]MeldHeap<(at: Int, cnt: Int)> = [V] v => MeldHeap.of([(at: bound[v], cnt: 1)])
var dsu = Dsu<V>()

// Raise F (the breakpoints of component `u`) to max(F, l - d) at the integers.
fn raise(u: V, d: Int):
    var (pos, slope, val) = (set[u].top!.at, 0, 0)   // F(pos) = val, slope just right of pos
    while let b = set[u].top, val + slope * (b.at - pos) < b.at - d:
        (val, pos, slope) = (val + slope * (b.at - pos), b.at, slope + b.cnt)
        set[u].pop()
    let (step, part) = (pos - val - d).divmod(slope - 1)
    let added = [
        (at: d, cnt: 1),
        (at: pos + step, cnt: slope - 1 - part),
        (at: pos + step + 1, cnt: part),
    ].filter(b => b.cnt > 0)
    set[u].push_all(added)

for (u, v, w) in edges.sorted(by: e => -e.w):
    let (kept, merged) = dsu.merge(u, v)!
    set[kept].meld(set[merged].take())
    bound[kept] += bound[merged] - 2 * w
    if bound[kept] < set[kept].top!.at: raise(kept, bound[kept])

let breaks = set[dsu.find(V.first!)].drain().flat_map(b => [b.at] * b.cnt)
let below = prefix(breaks)
let q: 1..=250_000 = read()
let asks: [0..q]1..=1e9 = read()
for l in asks:
    let k = breaks.count_below(l)
    print(l * k - below.prefix(k))

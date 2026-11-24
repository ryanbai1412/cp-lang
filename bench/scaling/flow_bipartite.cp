// let: V = \sqrt n; L = \sqrt n; R = \sqrt n; F = \sqrt n; P = \sqrt n
// fixed: c
use std.flow.{Matching, best_matching, stable_matching, edge_coloring}
// Bipartite and general matchings on sqrt n vertices a side and n edges.
let n: Int = read()
let k = n.isqrt
domain V = 0..k
domain L = 0..k
domain R = 0..k
domain E = 0..k * k
var s = 0
// Greedy matches l_i to r_i and strands l_{k-1}, whose one augmenting path walks the whole
// chain; the padding edges l_i - r_j, j < i, only lead back along it.
let badj = [L] l => do:
    let i = l.ord
    var b = if i + 1 < k: [R(i), R(i + 1)] else: [R(0)]
    b += [R(j) for j in 0..i if i + 1 < k or j > 0]
    b
let adj = [V] v => [V((v.ord * 31 + j * 7) % k) for j in 0..k]
for _ in 0..64:
    s += general_matching(adj).size // timed: flow.general_matching
for _ in 0..112:
    s += max_matching(badj).size // timed: flow.max_matching
    s += min_vertex_cover(badj).left.count(x => x) // timed: flow.min_vertex_cover
for _ in 0..64:
    var m = Matching<L, R>()
    for l in L:
        for r in badj[l]: m.add(l, r)
    s += m.augment_all() // timed: flow.Matching.augment_all
for _ in 0..16:
    s += max_weight_matching((l: L, r: R) => (l.ord * 7919 + r.ord * 31) % 1009).total // timed: flow.max_weight_matching
s += min_weight_matching((l: L, r: R) => (l.ord * 7919 + r.ord * 31) % 1009).total // timed: flow.min_weight_matching
let bip: [E](l: L, r: R, w: Int) = [E] e => (l: L(e.ord / k), r: R(e.ord % k), w: (e.ord * 7919) % 1009)
// Everyone ranks the other side alike, so l_i is turned down by r_0..r_{i-1}: L R proposals.
let pl = [L] _ => [R(j) for j in 0..k]
let pr = [R] _ => [L(j) for j in 0..k]
let ends: [E](l: L, r: R) = [E] e => (l: bip[e].l, r: bip[e].r)
s += (best_matching(bip))!.total // timed: flow.best_matching
s += edge_coloring(ends).colors // timed: flow.edge_coloring
for _ in 0..32:
    s += stable_matching(pl, pr).len // timed: flow.stable_matching
print(s)

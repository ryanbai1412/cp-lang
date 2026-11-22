// Codeforces 2062 G: Permutation Factory.
// View p as points (i, p[i]) and q as points (j, q[j]). Every swap of cost
// min(|i - j|, |p[i] - p[j]|) reduces total Manhattan distance to any target
// matching by at most 2, so half the minimum Manhattan matching cost is a lower
// bound. It is achieved in two stages: first sort positions i -> mate[i] by
// swaps reducing position distance, then sort values p[i] -> q[mate[i]] by
// swaps reducing value distance.
//
// In any nontrivial permutation, let r be the leftmost index with perm[r] < r,
// and l be the largest index < r in r's cycle. By minimality of r, perm[l] > l;
// by maximality of l in r's cycle, perm[r] <= l < r <= perm[l]. Swapping (l, r)
// therefore reduces sum |x - perm[x]| by exactly 2 * (r - l) — twice the swap
// cost — matching the Manhattan lower bound without re-sorting cycles.

use std.flow.min_weight_matching
use std.seq.{inverse, swap_at}

fn sort_perm<D>(mut perm: [D]D) -> List<(D, D)>:
    var ops: List<(D, D)> = []
    while let r = D.first(x => perm[x] < x):
        var l = perm[r]
        var cur = perm[l]
        while cur != r:
            if cur < r and cur > l: l = cur
            cur = perm[cur]
        ops.push((l, r))
        perm.swap_at(l, r)
    ops

@!multitest
let n: 2..=100 = read()
domain I = 1..=n
let p: [I]I = read()
let q: [I]I = read()

let (_, mate) = min_weight_matching<I, I>((i, j) => abs(i.ord - j.ord) + abs(p[i].ord - q[j].ord))

// Stage 1: move each element from i to mate[i].
var pos_perm = mate.copy()
var ops = sort_perm(mut pos_perm)

// Stage 2: value v now sits at pos_of[v] and must become q[pos_of[v]].
let inv_p = p.inverse()
var pos_of: [I]I = [I] v => mate[inv_p[v]]
var val_perm: [I]I = [I] v => q[pos_of[v]]
for (u, v) in sort_perm(mut val_perm):
    ops.push((pos_of[u], pos_of[v]))
    pos_of.swap_at(u, v)

print(ops.len)
for (u, v) in ops: print(u, v)

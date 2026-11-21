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
use std.seq.inverse

fn sort_perm<D>(mut perm: [D]D) -> List<(D, D)>:
    var ops: List<(D, D)> = []
    while let r = D.first(x => perm[x] < x):
        let l = perm.cycle(from: r).filter(x => x < r).max()!
        ops.push((l, r))
        perm.swap_at(l, r)
    ops

@!multitest
let n: 2..=100 = read()
domain I = 1..=n
let p: [I]I = read()
let q: [I]I = read()

let (_, mate) = min_weight_matching<I, I>((i, j) => abs(i - j) + abs(p[i] - q[j]))

// Stage 1: move each element from i to mate[i].
var moving = mate.clone()
var ops = sort_perm(mut moving)

// Stage 2: value v now sits at pos_of[v] and must become q[pos_of[v]].
let inv_p = p.inverse()
var pos_of = [I] v => mate[inv_p[v]]
for (u, v) in sort_perm([I] v => q[pos_of[v]]):
    ops.push((pos_of[u], pos_of[v]))
    pos_of.swap_at(u, v)

print(ops.len)
for (u, v) in ops: print(u, v)

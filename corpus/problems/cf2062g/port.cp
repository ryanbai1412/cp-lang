// Source: Codeforces 2062G (Permutation Factory), https://codeforces.com/contest/2062/problem/G
// limits: 2000 ms, 512 MB
// Problem: permutations p, q of n <= 100. Swapping p_i and p_j costs min(|i - j|, |p_i - p_j|);
//   print a cheapest sequence of at most n^2 swaps turning p into q. Multitest,
//   sum of n^3 <= 1e6.
// Solution: as points (i, p_i) a swap of cost c moves the matching to q's points by at most 2c
//   in Manhattan distance, so half the minimum Manhattan matching (Hungarian, O(n^3)) bounds
//   the cost. It is met in two stages: sort positions to their mates, then values, each by
//   swaps whose cost is half the distance they remove. O(n^3).
// In any nontrivial permutation, let r be the leftmost index with perm[r] < r,
// and l be the largest index < r in r's cycle. By minimality of r, perm[l] > l;
// by maximality of l in r's cycle, perm[r] <= l < r <= perm[l]. Swapping (l, r)
// therefore reduces sum |x - perm[x]| by exactly 2 * (r - l) — twice the swap
// cost — matching the Manhattan lower bound without re-sorting cycles.
// Bounds: n: 2..=100; p, q: 1..=n

use std.seq.{swap_at}

fn sort_perm<D>(mut perm: [D]D) -> List<(D, D)>:
    var ops: List<(D, D)> = []
    while let r = D.find(x => perm[x] < x):
        var l = perm[r]
        var cur = perm[l]
        while cur != r:
            if l < cur < r: l = cur
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
var pos_perm = mate.clone()
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

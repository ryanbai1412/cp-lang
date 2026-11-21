// Codeforces 2062 G: Permutation Factory.
// View p as points (i, p[i]) and q as points (j, q[j]). Every swap of cost
// min(|i - j|, |p[i] - p[j]|) reduces total Manhattan distance to any target
// matching by at most 2, so half the minimum Manhattan matching cost is a lower
// bound. It is always achieved in two stages: first sort positions i -> mate[i]
// by swaps reducing position distance, then sort values p[i] -> q[mate[i]] by
// swaps reducing value distance.
use std.flow.{max_weight_matching}
use std.seq.{swap_at}

fn sort_perm<D>(mut perm: [D]D) -> List<(D, D)>:
    var ops: List<(D, D)> = []
    for i in D:
        while perm[i] != i:
            var cyc: List<D> = []
            var cur = perm[i]
            while cur != i:
                cyc.push(cur)
                cur = perm[cur]
            cyc.sort()
            let r = first(cyc, x => perm[x] < x)!
            let l = first(cyc.rev, x => x < r) ?? i
            ops.push((l, r))
            perm.swap_at(l, r)
    ops

@!multitest
let n: 2..=100 = read()
domain I = 1..=n
let p: [I]I = read()
let q: [I]I = read()

let (_, mate) = max_weight_matching < I, I > (
    (i, j) => -(abs(i.ord - j.ord) + abs(p[i].ord - q[j].ord))
)

var pos_perm: [I]I = [I] i => mate[i]
var ops = sort_perm(mut pos_perm)

var val_perm: [I]I = I.first
var pos_of: [I]I = I.first
for i in I:
    val_perm[p[i]] = q[mate[i]]
    pos_of[p[i]] = mate[i]

for (u, v) in sort_perm(mut val_perm):
    ops.push((pos_of[u], pos_of[v]))
    pos_of.swap_at(u, v)

print(ops.len)
for (u, v) in ops:
    print(u, v)

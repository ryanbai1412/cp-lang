// Codeforces 2062 G: Permutation Factory (dream).
// View p as points (i, p[i]) and q as points (j, q[j]). Every swap of cost
// min(|i - j|, |p[i] - p[j]|) reduces total Manhattan distance to any target
// matching by at most 2, so half the minimum Manhattan matching cost is a lower
// bound. It is always achieved in two stages: first sort positions i -> mate[i]
// by swaps reducing position distance, then sort values p[i] -> q[mate[i]] by
// swaps reducing value distance.
with io(base: 1)

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
            let r = cyc.first(where: x => perm[x] < x)!
            let l = cyc.last(where: x => x < r) ?? i
            ops.push((l, r))
            perm.swap(l, r)
    ops

--- multitest ---
domain I = read(2..=100)
let p: [I]I = read()
let q: [I]I = read()

let (_, mate) = min_weight_matching<I, I>((i, j) => abs(i - j) + abs(p[i] - q[j]))

var ops = sort_perm([I] i => mate[i])

var val_perm: [I]I = [I] _ => I.first
var pos_of: [I]I = [I] _ => I.first
for i in I:
    val_perm[p[i]] = q[mate[i]]
    pos_of[p[i]] = mate[i]

for (u, v) in sort_perm(val_perm):
    ops.push((pos_of[u], pos_of[v]))
    pos_of.swap(u, v)

print(ops.len)
for (u, v) in ops: print(u, v)

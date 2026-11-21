// Codeforces 2062 G: Permutation Factory (dream).
// View p as points (i, p[i]) and q as points (j, q[j]). Every swap of cost
// min(|i - j|, |p[i] - p[j]|) reduces total Manhattan distance to any target
// matching by at most 2, so half the minimum Manhattan matching cost is a lower
// bound. It is always achieved in two stages: first sort positions i -> mate[i]
// by swaps reducing position distance, then sort values p[i] -> q[mate[i]] by
// swaps reducing value distance.
with io(base: 1)

// The editorial's cycle step: in the cycle through i, swap the first element r
// that must move left with the nearest l < r in the same cycle (i if none).
// Each swap shortens the remaining distance by exactly twice its cost.
fn sort_perm<D>(mut perm: [D]D) -> List<(D, D)>:
    var ops: List<(D, D)> = []
    for i in D:
        while perm[i] != i:
            let cyc = perm.cycle(from: i).sort()
            let r = cyc.first(x => perm[x] < x)!
            let l = cyc.last(x => x < r) ?? i
            ops.push((l, r))
            perm.swap(l, r)
    ops

@!multitest
domain I = read(2..=100)
let p: [I]I = read()
let q: [I]I = read()

let mate = min_weight_matching<I, I>((i, j) => abs(i - j) + abs(p[i] - q[j])).mate

// Stage 1 moves each element from i to mate[i].
var ops = sort_perm(mate)

// Stage 2: value v now sits at at[v] and must become q[at[v]].
var at = [I] v => mate[p.inv[v]]
for (u, v) in sort_perm([I] v => q[at[v]]):
    ops.push((at[u], at[v]))
    at.swap(u, v)

print(ops.len)
for (u, v) in ops: print(u, v)

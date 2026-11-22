// Codeforces 2211 E — Interactive Tree GCD Partition
//
// Vertices arrive bottom-up from n down to 1: each vertex `cur` has value `a`
// and children `children` (all > cur). We partition the subtree into the fewest
// connected components each having a common divisor > 1 (or singletons).
//
// Greedily, `good[v]` stores the LCM of candidate GCDs that v's open root
// component can share with its parent. At `cur`, any child `nxt` contributes
// `gcd(a, good[nxt])` without paying an extra cut, so we combine
// `good[cur] = lcm(good[cur], gcd(a, good[nxt]))` and sum child costs.
// If `good[cur] == 1`, no child component can absorb `cur`, so `cur` opens a
// new component (`cost[cur] += 1`, `good[cur] = a`).

@!interactive
@!multitest
let n: 1..=200_000 = read()
domain V = 1..=n
var cost: [V]Int = 0
var good: [V]Int = 1
for cur in V.rev:
    let (a, k): (1..=10 ** 18, 0..=n) = read()
    let children: [0..k]V = read()
    for nxt in children:
        cost[cur] += cost[nxt]
        good[cur] = lcm(good[cur], gcd(a, good[nxt]))
    if good[cur] == 1:
        cost[cur] += 1
        good[cur] = a
    print(cost[cur])

// Codeforces 2062 F, tuned: port.cp's DP over roles written out as orig.cpp
// does, a flat `Int` row per count with a sentinel for "unreached" and the
// nine role moves spelled out, instead of `?Int` cells and a list of roles.
@!multitest
let n: 2..=3000 = read()
const INF = 1_000_000_000_000_000_000
var cost: [0..=n, 0..5]Int = INF
cost[0, 0] = 0
for (i, (a, b)) in read<[n](a: 0..=1e9, b: 0..=1e9)>().sorted(by: c => c.a - c.b).items:
    unchecked:
        for k in (0..=i).rev:
            let c0 = cost[k, 0]
            if c0 != INF:
                cost[k + 1, 1] min= c0 + 2 * b
                cost[k + 1, 2] min= c0 + b
            let c1 = cost[k, 1]
            if c1 != INF:
                cost[k + 1, 1] min= c1 + a + b
                cost[k + 1, 2] min= c1 + a
            let c2 = cost[k, 2]
            if c2 != INF:
                cost[k + 1, 2] min= c2 + a + b
                cost[k + 1, 3] min= c2 + b
                cost[k + 1, 4] min= c2 + a
            let c3 = cost[k, 3]
            if c3 != INF:
                cost[k + 1, 3] min= c3 + a + b
                cost[k + 1, 4] min= c3 + 2 * a
print([cost[k, 4] for k in 2..=n])

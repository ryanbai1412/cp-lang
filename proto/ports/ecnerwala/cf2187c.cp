// Codeforces 2187 C: greedily take the farthest forward edge at each vertex,
// then merge depth-indexed (count, subtree-sum) tables bottom-up (small-to-large).
@!multitest
let (n, m): (Int, Int) = read()
var nxt: [0..n]Int = [0..n] i => i + 1
let edges: [0..m](Int, Int) = read()
for (u, v) in edges: nxt[u - 1] max= v - 1

var sz: [0..n]Int = 0
var by_depth: [0..n]List<(cnt: Int, sum: Int)> = []
var ans = 0

for i in 0..n:
    sz[i] += 1
    by_depth[i].push((1, sz[i]))
    if i < n - 1:
        let p = nxt[i]
        sz[p] += sz[i]
        if by_depth[p].len < by_depth[i].len:
            assert p != i
            swap(mut by_depth[p], mut by_depth[i])
        let lp = by_depth[p].len
        let li = by_depth[i].len
        for l in 1..=li:
            let (af, as2) = by_depth[p][lp - l]
            let (bf, bs2) = by_depth[i][li - l]
            ans += (af * bs2 + bf * as2) * l
            by_depth[p][lp - l] = (af + bf, as2 + bs2)
        by_depth[i] = []

print(ans)

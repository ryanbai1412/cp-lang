// 1950G Shuffling Songs: reachable (mask, last) over a compatibility graph.
@!multitest
let n: 1..=16 = read()
let s = read<[n](g: Str, w: Str)>()
let fits = [[s[i].g == s[j].g or s[i].w == s[j].w for j in 0..n] for i in 0..n]
var dp: [1 << n, n]Bool = false
for i in 0..n:
    dp[1 << i, i] = true
var best = 0
for mask in 1..1 << n:
    for last in 0..n:
        if not dp[mask, last]: continue
        best max= (0..n).count(i => mask.bit(i))
        for j in 0..n:
            if not mask.bit(j) and fits[last][j]:
                dp[mask | (1 << j), j] = true
print(n - best)

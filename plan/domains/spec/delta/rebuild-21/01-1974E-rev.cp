// CF 1974E, revised: Charlie may make at most q purchases in total (input "m x q", q <= 10;
// sum of h <= 1e4). The dp gains a purchase-count dimension.
@!multitest
let m = read<1..=50>()
let x = read<1..=1e8>()
let q = read<0..=10>()
let offer = read<[m](c: 0..=1e8, h: 1..=1e3)>()
let hs = offer.map(o => o.h).sum()

// dp[i, j, s]: least money spent on months before cut i, with j purchases, for happiness s
var dp: [0..=m, 0..=q, 0..=hs]?Int = None
dp[0, 0, 0] = 0
for i in 1..=m:
    let o = offer[i - 1]
    for j in 0..=q:
        for s in 0..=hs:
            dp[i, j, s] = dp[i - 1, j, s]
            if j >= 1 and s >= o.h:
                if let p = dp[i - 1, j - 1, s - o.h]:
                    if p + o.c <= x * (i - 1):
                        dp[i, j, s] min= p + o.c
print((0..=hs).filter(s => (0..=q).any(j => dp[m, j, s] != None)).max()!)

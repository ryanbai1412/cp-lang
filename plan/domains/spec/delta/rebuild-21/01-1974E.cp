// CF 1974E Money Buys Happiness: dp over (month cut, happiness) -> least money spent.
@!multitest
let m = read<1..=50>()
let x = read<1..=1e8>()
let offer = read<[m](c: 0..=1e8, h: 1..=1e3)>()
let hs = offer.map(o => o.h).sum()

// dp[i, s]: least money spent on months before cut i to collect happiness s
var dp: [0..=m, 0..=hs]?Int = None
dp[0, 0] = 0
for i in 1..=m:
    let o = offer[i - 1]
    for s in 0..=hs:
        dp[i, s] = dp[i - 1, s]
        if s >= o.h:
            if let p = dp[i - 1, s - o.h]:
                if p + o.c <= x * (i - 1):
                    dp[i, s] min= p + o.c
print((0..=hs).filter(s => dp[m, s] != None).max()!)

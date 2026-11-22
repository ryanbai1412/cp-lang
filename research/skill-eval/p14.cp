let (n, x) = read<(1..=20, 1..=1e9)>()
domain P = 0..n
let w = read<[P]1..=x>()
var dp: [Subset<P>]?(Int, Int) = None
var ans = 0
for mask in Subset<P>:
    if mask.len == 0:
        dp[mask] = (1, 0)
    if let (rides, load) = dp[mask]:
        if mask.len == n:
            ans = rides
        for p in P:
            if p not in mask:
                let nxt = mask.with(p)
                let cand = if load + w[p] <= x: (rides, load + w[p]) else: (rides + 1, w[p])
                dp[nxt] min= cand
print(ans)

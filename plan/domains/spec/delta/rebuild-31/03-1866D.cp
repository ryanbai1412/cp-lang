// CF 1866D Digital Wallet
// Operation p takes from columns p..p+k-1, so after column c the number of
// operations used, j, satisfies c-k+1 <= j <= c (Hall on intervals). Each
// column contributes its t largest values; dp[j] is updated in place with
// j descending, so it still holds the previous column's value when read.
let (n, m, k): (1..=10, 1..=1e5, 1..=min(10, m)) = read()
domain R = 0..n
domain Col = 1..=m
let a = read<[R, Col]1..=1e6>()
let ops = m - k + 1
domain Ops = 0..=ops
var dp: [Ops]?Int = None
dp[Ops(0)] = 0
for c in Col:
    let top = [a[r, c] for r in R].sorted(by: desc).scan(+)
    let hi = min(Int(c) - 1, ops)
    for j in (max(0, Int(c) - k)..=hi).rev:
        if let v = dp[Ops(j)]:
            for t in 1..=min(n, Int(c) - j, ops - j):
                dp[Ops(j + t)] max= v + top[t]
    if Int(c) - k >= 0: dp[Ops(Int(c) - k)] = None
print(dp[Ops(ops)]!)

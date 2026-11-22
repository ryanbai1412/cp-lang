// p14 on Int masks (row 12).
let (n, x): (1..=20, 1..=1e9) = read()
let w = read<[n]1..=1e9>()
let full = 1 << n
var dp: [0..full]?(Int, Int) = None
dp[0] = (1, 0)
for mask in 0..full:
    let (rides, last) = dp[mask]!
    for p in 0..n if mask >> p & 1 == 0:
        dp[mask | 1 << p] min= if last + w[p] <= x: (rides, last + w[p]) else: (rides + 1, w[p])
print(dp[full - 1]!.0)

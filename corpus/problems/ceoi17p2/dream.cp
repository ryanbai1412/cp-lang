// CEOI 2017 Sure Bet: bet one unit on any subset of outcomes of two kinds with odds a[i] and b[j];
// guarantee the most profit. Taking odds best-first is optimal for each side, and with k bets
// placed the profit is min(sum a, sum b) - k: always extend the poorer side.
let n: 1..=1e5 = read()
let bets: [n](a: F64, b: F64) = read()
let a = bets.map(o => o.a).sorted(by: desc)
let b = bets.map(o => o.b).sorted(by: desc)
var (sa, sb, i, j, best) = (0.0, 0.0, 0, 0, 0.0)
while i < n or j < n:
    if j == n or (i < n and sa <= sb):
        sa += a[i]
        i += 1
    else:
        sb += b[j]
        j += 1
    best max= min(sa, sb) - (i + j)
print(best, precision: 4)

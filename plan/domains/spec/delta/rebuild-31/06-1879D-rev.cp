// CF 1879D Sum of XOR Functions, revised: weight (r - l + 1)^2.
// (r - l)^2 = r^2 - 2 r l + l^2, so each parity also keeps the sum of l^2.
type Z = Mod<998244353>
let n = read<1..=3e5>()
domain P = 0..n
let a = read<[P]0..=1e9>()
let px = a.scan(^)
var ans: Z = 0
for b in 0..30:
    var cnt = [0, 0]
    var sum = [0, 0]
    var sq = [0, 0]
    for c in P.cuts:
        let bit = (px[c] >> b) & 1
        let o = 1 - bit
        let r = Int(c)
        ans += Z(1 << b) * (Z(cnt[o]) * Z(r * r) - Z(2 * r) * Z(sum[o]) + Z(sq[o]))
        cnt[bit] += 1
        sum[bit] += r
        sq[bit] += r * r
print(ans)

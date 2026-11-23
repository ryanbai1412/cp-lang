// CF 1879D Sum of XOR Functions
// Bit by bit: f(l, r) has bit b iff the prefix xors at cuts l and r differ
// in bit b, and the weight is r - l. For each cut keep, per parity of bit
// b, how many earlier cuts there are and the sum of their indices.
type Z = Mod<998244353>
let n = read<1..=3e5>()
domain P = 0..n
let a = read<[P]0..=1e9>()
let px = a.scan(^)
var ans: Z = 0
for b in 0..30:
    var cnt = [0, 0]
    var sum = [0, 0]
    for c in P.cuts:
        let bit = (px[c] >> b) & 1
        let o = 1 - bit
        ans += Z(1 << b) * Z(cnt[o] * Int(c) - sum[o])
        cnt[bit] += 1
        sum[bit] += Int(c)
print(ans)

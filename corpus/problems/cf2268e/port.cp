// Codeforces 2268 E — Bitwise Catalan Autocorrelation
//
// Values satisfy `0 <= xs[i] < 2^BITS` (`BITS = 18`), so each bit `b in 0..BITS`
// contributes independently (`cur * (1 << b)`). Let `a[i] in {-1, +1}` be the
// prefix XOR parity of bit `b` (`a[0] = 1`, flipping sign when `xs[i].bit(b)`).
//
// Across all `C_n = comb.catalan(n)` binary trees on `n` leaves (each having
// `n - 1` internal merges), the base sum is `C_n * (n - 1)`. When the total
// bit-`b` parity is even (`a[n] == 1`), each gap `d in 1..n` appears in
// `weight[d] = C_d * C_{n - d}` trees and loses `sum_i a[i] * a[i + d] = corr[n - d]`,
// computed as the autocorrelation `convolve(a, a.rev)` via NTT in O(n log n).

const MOD = 998_244_353
const BITS = 18
const MAX_X = (1 << BITS) - 1
type Z = Mod<MOD>
let comb = Comb<Z>()

@!multitest
let n: 1..=200_000 = read()
let xs: [1..=n]0..=MAX_X = read()
let weight = [0..n] d => if d == 0: Z(0) else: comb.catalan(d) * comb.catalan(n - d)
var ans: Z = 0
for b in 0..BITS:
    var a: [0..=n]Z = 1
    for i in 1..=n: a[i] = if xs[i].bit(b): -a[i - 1] else: a[i - 1]
    var cur = comb.catalan(n) * (n - 1)
    if a[n] == 1:
        let corr = convolve(a, [a[n - i] for i in 0..=n])
        for d in 1..n: cur -= weight[d] * corr[n - d]
    ans += cur * (1 << b)
print(ans)

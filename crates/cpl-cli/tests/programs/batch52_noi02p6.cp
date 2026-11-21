// NOI '02 P6 (Robot M): given prime factorization N = prod p_i^e_i, count independent,
// odd-prime-parity, and dependent room numbers modulo composite 10000.
// Uses v2.2 composite-modulus Mod<10_000> (.pow(e) + automatic normalization) and
// Bool-indexed parity DP ([Bool]Z).
// original: corpus/submissions/3754486.cpp

const MOD = 10_000
type Z = Mod<MOD>

let n: Int = read()
var dp: [Bool]Z = [1, 0]
var total: Z = 1

for _ in 0..n:
    let (p, e): (Int, Int) = read()
    total *= Z(p) ** e
    if p > 2:
        let (even, odd) = (dp[false], dp[true])
        dp[false] = even + odd * (p - 1)
        dp[true] = odd + even * (p - 1)

print(dp[false] - 1)
print(dp[true])
print(total - dp[false] - dp[true])

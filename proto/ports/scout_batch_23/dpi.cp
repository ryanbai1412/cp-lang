// AtCoder EDPC I - Coins (DMOJ: dpi, TL 1.0s, ML 1024MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// N (odd, 1 <= N <= 2999) independent coins; coin i lands heads with probability p_i.
// Compute the probability of getting strictly more heads than tails, to 10^-9 precision.
//
// Algebraically, the distribution of heads is the polynomial product
//   P(x) = prod_{i=1..N} ((1 - p_i) + p_i * x)
// and the answer is sum_{k > N/2} [x^k] P(x).
let n: 1..=2999 = read() where n % 2 == 1
domain Coin = 0..n
// WISH MG11: floating-point range type `0.0..=1.0` for probability input & cpl stress
let p: [Coin]F64 = read()

// WISH MG6: `Poly<F64>.product([(1.0 - p[i], p[i]) for i in Coin])` (divide-and-conquer
// FFT product in O(N log^2 N)) or linear DP in O(N^2):
var dp = List.filled(n + 1, 0.0)
dp[0] = 1.0
for i in Coin:
    let pi = p[i]
    let qi = 1.0 - pi
    for h in (0..=i.to_idx).rev:
        dp[h + 1] += dp[h] * pi
        dp[h] *= qi

var ans = 0.0
for h in (n / 2 + 1)..=n:
    ans += dp[h]

// WISH MG9: fixed-precision float format `{ans:.10f}` (silently dropped by proto/cpi)
print("{ans:.10f}")

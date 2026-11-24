// Source: Codeforces 2159 E, Super-Short-Polynomial-San,
//   https://codeforces.com/contest/2159/problem/E
// limits: 7000 ms, 1024 MB
// Problem: a, b, c in 1..1e9+6; q <= 3e5 online queries (n, k), each xored with the previous
//   answer, 0 <= n <= 3e5, 0 <= k <= 2n: print the sum of the coefficients of x^0..x^k in
//   (a x^2 + b x + c)^n modulo 1e9+7.
// Solution: Split n = i * B + j with B = 1200, so p^n = p^(iB) * p^j, and the answer is the sum
//   over z <= min(2j, k) of [x^z] p^j times a prefix sum of p^(iB) up to min(2iB, k - z).
//   Because p has degree 2, p * f' = m p' f gives every coefficient of f = p^m in O(m). The
//   tables of p^j and of prefix sums of p^(iB) are filled lazily: O(N^2 / B + Q B).

type Z = Mod<1_000_000_007>
const N = 300_000
const B = 1200

let (a, b, c): (Z, Z, Z) = read()
let comb = Comb<Z>()
let inv_c = 1 / c

fn coeffs_of_power(m: Int) -> List<Z>:
    if m == 0: return [Z(1)]
    var f: List<Z> = [c ** m, m * b * c ** (m - 1)]
    for z in 2..=2 * m:
        let num = b * f[z - 1] * (m - z + 1) + a * f[z - 2] * (2 * m - z + 2)
        f.push(num * comb.inv(z) * inv_c)
    f

// Lazily filled tables (`lazy[...]` in dream:25-26 is not yet supported, and
// `?List<Z>` unwraps copy the 600k-element vector on every query).
var small: [0..B]List<Z> = []
var big: [0..=N / B]List<Z> = []

var ans: Z = 0
repeat read():
    let (n, k): (0..=N, 0..=2 * N) = (read<Int>() ^ ans.v, read<Int>() ^ ans.v)
    let (i, j) = (n / B, n % B)
    if small[j].len == 0: small[j] = coeffs_of_power(j)
    if big[i].len == 0:
        let coef = coeffs_of_power(i * B)
        let p = coef.scan(+)
        big[i] = [p.prefix(t + 1) for t in 0..coef.len]
    ans = sum(small[j][z] * big[i][min(2 * i * B, k - z)] for z in 0..=min(2 * j, k))
    print(ans)

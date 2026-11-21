// Codeforces 2159 E: Super-Sum.
// Online queries for the sum of the first `k + 1` coefficients of `p^n`,
// `p(x) = a * x^2 + b * x + c`, with inputs xored by the previous answer.
// Split `n = i * B + j` with `B = 1200` (~sqrt(N) balancing O((N / B) * N)
// big-table work against O(Q * B) query work for N, Q <= 3e5):
//   p^n = p^(i * B) * p^j,
//   answer(n, k) = sum_{z=0}^{min(2j, k)} [x^z] p^j * prefix(p^(i * B))[k - z].
// Because p has degree 2, `p * f' = m * p' * f` gives every coefficient of
// `f = p^m` in O(m) total time (faster than O(m log m) NTT):
//   c * z * f[z] = b * (m - z + 1) * f[z - 1] + a * (2m - z + 2) * f[z - 2].

type Z = Mod<1_000_000_007>
const N = 300_000
const B = 1200

let (a, b, c): (1..< Z.MOD, 1..< Z.MOD, 1..< Z.MOD) = read()

fn coeffs_of_power(m: Int) -> List<Z>:
    if m == 0: return [Z(1)]
    var f: List<Z> = [c ** m, m * b * c ** (m - 1)]
    for z in 2..=2 * m:
        f.push((b * f[z - 1] * (m - z + 1) + a * f[z - 2] * (2 * m - z + 2)) / (c * z))
    f

let small = lazy[0..B] j => coeffs_of_power(j)
let big = lazy[0..=N / B] i => coeffs_of_power(i * B).scan(+)

var ans: Z = 0
repeat read():
    let (n, k): (0..=N, Int) = read(xor: ans)
    let (i, j) = n.divmod(B)
    ans = sum(small[j][z] * big[i][min(2 * i * B, k - z)] for z in 0..=min(2 * j, k))
    print(ans)

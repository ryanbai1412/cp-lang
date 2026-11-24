// Codeforces 2159 E (dream). Each query asks for the sum of the first k + 1
// coefficients of p^n, p = a·x² + b·x + c, online (inputs xored with the last answer).
// Split n = i·B + j, so p^n = p^(i·B) · p^j: keep p^j for every j < B and the prefix
// sums of p^(i·B) for every i, and
//   answer(n, k) = Σ_z [x^z] p^j · prefix(p^(i·B))[k − z].
// The coefficients f of p^m come from p · f' = m · p' · f, which reads
//   c·z·f[z] = b·(m − z + 1)·f[z − 1] + a·(2m − z + 2)·f[z − 2].
type Z = Mod<1e9 + 7>
// B balances the O(N^2 / B) table of big powers against O(B) work per query.
const N = 3e5, B = 1200

let (a, b, c): (Z, Z, Z) = read()

fn coeffs_of_power(m: Int) -> List<Z>:
    var f: List<Z> = [c ** m]
    for z in 1..=2 * m:
        let two_back = f.get(z - 2) ?? 0          // no coefficient below x^0
        f.push((b * f[z - 1] * (m - z + 1) + a * two_back * (2 * m - z + 2)) / (c * z))
    f

let small = [0..B] j => coeffs_of_power(j)
let big = [0..=N / B] i => coeffs_of_power(i * B).scan(+)

var ans: Z = 0
repeat read():
    let (n, k): (0..=N, Int) = (read<Int>() ^ ans.v, read<Int>() ^ ans.v)
    let (i, j) = n.divmod(B)
    ans = sum(small[j][z] * big[i].prefix(min(2 * i * B, k - z) + 1) for z in 0..=min(2 * j, k))
    print(ans)

type Z = Mod<1_000_000_007>
const N = 300_000
const B = 1200

let (a, b, c): (Z, Z, Z) = read()
let comb = Comb<Z>()
let inv_c = 1 / c

// coefficients of p^n, from p * f' = n * p' * f: two terms per coefficient
fn coeffs(n: Int) -> List<Z>:
    var f = [c.pow(n)]
    for z in 1..=2 * n:
        var acc = f[z - 1] * b * (n - (z - 1))
        if z >= 2: acc += f[z - 2] * a * (2 * n - (z - 2))
        f.push(acc * comb.inv(z) * inv_c)
    f

// p^n = p^j * p^(iB) for n = i * B + j: small powers whole, large ones as prefix sums
var small: [0..B]List<Z> = []
var big: [0..=N / B]List<Z> = []

var ans: Z = 0
repeat read():
    let (n_enc, k_enc): (Int, Int) = read()
    let (n, k) = (n_enc ^ ans.v, k_enc ^ ans.v)
    let (i, j) = (n / B, n % B)
    if small[j].len == 0: small[j] = coeffs(j)
    if big[i].len == 0: big[i] = scan(coeffs(i * B), +)
    ans = sum(small[j][z] * big[i][min(2 * i * B, k - z)] for z in 0..=min(2 * j, k))
    print(ans)

type Z = Mod<1e9+7>
const N = 3e5, B = 1200

let (a, b, c) = read<(1..<Z.MOD, 1..<Z.MOD, 1..<Z.MOD)>()
let p = Poly<Z>([c, b, a])

let small = lazy [0..B] j => p ^ j
// prefix sums of the coefficients of p^n, from p * f' = n * p' * f
let big = lazy [0..=N / B] i =>
    let n = i * B
    var f = [c ^ n]
    for z in 1..=2 * n:
        f.push((b * f[z - 1] * (n - z + 1) + a * f.get(z - 2, 0) * (2 * n - z + 2)) / (z * c))
    prefix(f)

var ans: Z = 0
repeat read():
    let (n, k) = read<(0..=N, Int)>(xor: ans)
    let (i, j) = n.divmod(B)
    ans = sum(small[j][z] * big[i].upto(k - z + 1) for z in 0..=min(2 * j, k))
    print(ans)

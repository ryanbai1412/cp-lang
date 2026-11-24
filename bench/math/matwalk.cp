// Matched benchmark: atcoder_dp_r's walk count, a 50x50 Mat<Mod<1e9+7>> ** 10^18 run 20
// times: a product's `out + f * b` must cost one `%`, like the C++ `(ret + a * b) % mod`.

type Z = Mod<1_000_000_007>
let n = 50
var acc = Z(0)
for t in 0..20:
    var s = 7 + t
    var d = List.filled(n * n, Z(0))
    for i in 0..n * n:
        s = (s * 1103515245 + 12345) & 2_147_483_647
        d[i] = Z(s >> 30 & 1)
    let r = Mat<Z>.from_flat(n, n, d) ** 1_000_000_000_000_000_000
    for i in 0..n:
        for j in 0..n: acc += r.get(i, j)
print(acc)

// Matched benchmark: 80x80 Mat<Mod<998244353>> ** 10^18.

type Z = Mod<998_244_353>

let n = 80
var data = List.filled(n * n, Z(0))
var s = 42
unchecked:
    for i in 0..n * n:
        s = (s * 1103515245 + 12345) & 2_147_483_647
        data[i] = Z(s)

let m = Mat<Z>.from_flat(n, n, data)
let r = m ** 1_000_000_000_000_000_000
var acc = Z(0)
unchecked:
    for i in 0..n:
        for j in 0..n: acc = acc + r.get(i, j) * Z(i * n + j + 1)
print(acc.v)

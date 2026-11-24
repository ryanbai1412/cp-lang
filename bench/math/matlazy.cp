// Matched benchmark: a 64x64 Mat<Mod<998244353>> product chain, against C++ that sums each row's
// products unreduced in 64 bits and reduces every few steps, as a fast hand-written product does.

type Z = Mod<998_244_353>
let n = 64
var s = 7
var d = List.filled(n * n, Z(0))
for i in 0..n * n:
    s = (s * 1103515245 + 12345) & 2_147_483_647
    d[i] = Z(s)
let x = Mat<Z>.from_flat(n, n, d.clone())
var m = Mat<Z>.from_flat(n, n, d)
for t in 0..400: m = m * x
var acc = Z(0)
for i in 0..n:
    for j in 0..n: acc += m.get(i, j)
print(acc)

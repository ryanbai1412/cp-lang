// Matched benchmark: 8 passes of `m.at(r, c) += a.get(r, c) + k` over 2000x2000 Mat<Mod>, the
// way noi21p2 counts paths: a settled Mat's `at` must cost an index, nothing more.

type Z = Mod<998_244_353>
let n = 2000
let a = Mat<Z>.identity(n)
var m = Mat<Z>.zeros(n, n)
for k in 0..8:
    for r in 0..n:
        for c in 0..n:
            m.at(r, c) += a.get(r, c) + Z(k)
var s = Z(0)
for r in 0..n:
    s += m.get(r, r)
print(s)

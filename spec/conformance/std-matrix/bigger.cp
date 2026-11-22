// expect: ok, output "YES\nYES\nYES\nYES\nYES"
use std.matrix.{Mat}
use std.mod.{Mod}
type Z = Mod<998244353>
let m = Mat<Z>.from_rows([[Z(2), Z(7), Z(1), Z(8)], [Z(2), Z(8), Z(1), Z(8)], [Z(3), Z(1), Z(4), Z(1)], [Z(5), Z(9), Z(2), Z(6)]])
let cp = m.char_poly()
var acc = Mat<Z>.zeros(4, 4)
var pw = Mat<Z>.identity(4)
for i in 0..5:
    acc = acc + Mat<Z>.from_flat(4, 4, [cp[i] * x for x in pw.data])
    pw = pw * m
print(acc == Mat<Z>.zeros(4, 4))
print(cp[0] == m.det())
print(m * m.adjugate() == Mat<Z>.from_flat(4, 4, [m.det() * x for x in Mat<Z>.identity(4).data]))
let s = Mat<Z>.from_rows([[Z(0), Z(1), Z(2), Z(3)], [Z(0), Z(0), Z(4), Z(5)], [Z(0), Z(0), Z(0), Z(6)], [Z(0), Z(0), Z(0), Z(0)]])
var full = Mat<Z>.zeros(4, 4)
for i in 0..4:
    for j in i + 1..4:
        full.set(i, j, s.get(i, j))
        full.set(j, i, Z(0) - s.get(i, j))
print(s.pfaffian() * s.pfaffian() == full.det())
let sing = Mat<Z>.from_rows([[Z(1), Z(2), Z(3)], [Z(2), Z(4), Z(6)], [Z(1), Z(0), Z(1)]])
print(sing * sing.adjugate() == Mat<Z>.zeros(3, 3) and sing.adjugate() != Mat<Z>.zeros(3, 3))

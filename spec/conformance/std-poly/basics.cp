// expect: ok, output "YES YES YES\nYES YES YES\nYES YES YES\nYES YES YES\nYES YES"
use std.poly.{Poly}
type Z = Mod<998244353>
print(Poly<Int>([1, 2, 0]).len == 3, Poly<Z>.from_int(5).at(0) == Z(5), Poly<Int>.zero.deg() == -1 and Poly<Int>.zero.len == 0)
print(Poly<Int>.one.at(0) == 1 and Poly<Int>.one.deg() == 0, Poly<Int>.x.deg() == 1 and Poly<Int>.x.at(1) == 1, Poly<Int>([1, 2, 0]).deg() == 1 and Poly<Int>([0, 0]).deg() == -1)
var p = Poly<Int>([1])
p.set(3, 7)
print(Poly<Int>([0, 0]).is_zero() and not Poly<Int>.one.is_zero(), Poly<Int>([1, 2]).at(1) == 2 and Poly<Int>([1, 2]).at(5) == 0, p.len == 4 and p.at(2) == 0 and p.at(3) == 7)
print(Poly<Z>([1, 2, 3]).deriv().at(1) == Z(6), Poly<Z>([1, 2, 3]).integral().at(3) == Z(1), Poly<Int>([1, 2, 3]).truncated(2).len == 2 and Poly<Int>([1]).truncated(3).at(2) == 0)
let s = Poly<Int>([1, 2]) + Poly<Int>([0, 0, 5])
let d = Poly<Int>([1, 2]) - Poly<Int>([1, 2, 3])
print(Poly<Int>([1, 2, 3]).eval(2) == 17, s.c == [1, 2, 5] and d.c == [0, 0, -3] and (-s).c == [-1, -2, -5])

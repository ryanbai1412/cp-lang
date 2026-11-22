// expect: ok, output "YES YES YES\nYES YES YES\nYES YES YES\nYES YES YES\nYES YES"
use std.poly.{Poly}
type Z = Mod<998244353>
print(yes(Poly<Int>([1, 2, 0]).len == 3), yes(Poly<Z>.from_int(5).at(0) == Z(5)), yes(Poly<Int>.zero.deg() == -1 and Poly<Int>.zero.len == 0))
print(yes(Poly<Int>.one.at(0) == 1 and Poly<Int>.one.deg() == 0), yes(Poly<Int>.x.deg() == 1 and Poly<Int>.x.at(1) == 1), yes(Poly<Int>([1, 2, 0]).deg() == 1 and Poly<Int>([0, 0]).deg() == -1))
var p = Poly<Int>([1])
p.set(3, 7)
print(yes(Poly<Int>([0, 0]).is_zero() and not Poly<Int>.one.is_zero()), yes(Poly<Int>([1, 2]).at(1) == 2 and Poly<Int>([1, 2]).at(5) == 0), yes(p.len == 4 and p.at(2) == 0 and p.at(3) == 7))
print(yes(Poly<Z>([1, 2, 3]).deriv().at(1) == Z(6)), yes(Poly<Z>([1, 2, 3]).integral().at(3) == Z(1)), yes(Poly<Int>([1, 2, 3]).truncated(2).len == 2 and Poly<Int>([1]).truncated(3).at(2) == 0))
let s = Poly<Int>([1, 2]) + Poly<Int>([0, 0, 5])
let d = Poly<Int>([1, 2]) - Poly<Int>([1, 2, 3])
print(yes(Poly<Int>([1, 2, 3]).eval(2) == 17), yes(s.c == [1, 2, 5] and d.c == [0, 0, -3] and (-s).c == [-1, -2, -5]))

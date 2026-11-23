// expect: ok, output "YES\nYES\nYES\nYES"
use std.ntt.{ntt}
type Z = Mod<17>
var a = [Z(1), Z(0), Z(0), Z(0)]
ntt(mut a)
print(yes(a == [Z(1), Z(1), Z(1), Z(1)]))
var b = [Z(1), Z(2), Z(3), Z(4)]
ntt(mut b)
ntt(mut b, inverse: true)
print(yes(b == [Z(1), Z(2), Z(3), Z(4)]))
var c = [Z(5), Z(6)]
ntt(mut c, bit_reversed: true)
ntt(mut c, inverse: true, bit_reversed: true)
print(yes(c == [Z(5), Z(6)]))
type Q = Mod<998244353>
var d: List<Q> = [Q(3), Q(1), Q(4), Q(1), Q(5), Q(9), Q(2), Q(6)]
ntt(mut d)
ntt(mut d, inverse: true)
print(yes(d == [Q(3), Q(1), Q(4), Q(1), Q(5), Q(9), Q(2), Q(6)]))

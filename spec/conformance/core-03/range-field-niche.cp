// expect: ok, output "18446744073709551615 3"
// rule: 03.2.3
struct H: h: U64, len: 0..=1e6
var x: ?H = None
x = H(18446744073709551615, 3)
print(x!.h, x!.len)

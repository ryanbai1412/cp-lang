// expect: ok, output "10 -2 -4 0\n1 2 3 4\n1 3 4 10"
// rule: std.ntt.fwht std.ntt.sos
// A table over a mask domain is the natural shape for a subset transform: `fwht` and `sos` take `[Mask]T` like a `List`, position `m` holding mask `m`.
use std.ntt.{fwht}
domain Mask = 0..4
var a: [Mask]Int = [1, 2, 3, 4]
fwht(mut a)
print(a)
fwht(mut a, inverse: true)
print(a)
sos(mut a)
print(a)

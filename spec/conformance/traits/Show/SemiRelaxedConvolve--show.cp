// expect: ok, output "2 5"
use std.ntt.{SemiRelaxedConvolve}
type Z = Mod<998244353>
var sc = SemiRelaxedConvolve<998244353>([Z(1), Z(1)])
sc.push(Z(2))
sc.push(Z(3))
print(sc)

// expect: ok, output "1 3 6"
use std.ntt.{RelaxedConvolve}
type Z = Mod<998244353>
var rc = RelaxedConvolve<998244353>()
rc.push(Z(1), Z(1))
rc.push(Z(2), Z(1))
rc.push(Z(3), Z(1))
print(rc)

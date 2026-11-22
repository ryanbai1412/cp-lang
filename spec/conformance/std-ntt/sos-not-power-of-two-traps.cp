// expect: R0401 at 3 (checked)
use std.ntt.{sos}
var a = [1, 2, 3]
sos(mut a)
print(a)

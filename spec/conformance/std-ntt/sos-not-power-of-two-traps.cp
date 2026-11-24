// expect: R0401 (checked)
use std.ntt.{sos}
var a = [1, 2, 3]
sos(mut a) // <- here
print(a)

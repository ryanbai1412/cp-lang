// expect: ok, output "5"
use std.ntt.{sos}
var a = [5]
sos(mut a)
print(a)

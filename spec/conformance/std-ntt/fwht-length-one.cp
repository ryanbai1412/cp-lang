// expect: ok, output "7"
use std.ntt.{fwht}
var a = [7]
fwht(mut a)
print(a)

// expect: R0401 at 3 (checked)
use std.ntt.{fwht}
var a = [1, 2]
fwht(mut a, inverse: true)
print(a)

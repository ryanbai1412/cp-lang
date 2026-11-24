// expect: R0401 (checked)
use std.ntt.{fwht}
var a = [1, 2]
fwht(mut a, inverse: true) // <- here
print(a)

// expect: R0401 at 3 (checked)
use std.ntt.{sos}
var a: List<Int> = []
sos(mut a)
print(a.len)

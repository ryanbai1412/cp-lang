// expect: R0401 (checked)
use std.ntt.{sos}
var a: List<Int> = []
sos(mut a) // <- here
print(a.len)

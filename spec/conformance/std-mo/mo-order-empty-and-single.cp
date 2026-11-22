// expect: ok, output "0 1"
use std.mo.{mo_order}
let none: List<Range<Int>> = []
print(mo_order(none).len, mo_order([2..5]).len)

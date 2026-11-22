// expect: ok, output "0 1"
use std.mo.{mo_order}
let empty: List<Range<Int>> = []
print(mo_order(empty).len, mo_order([2..5]).len)

// expect: ok, output "YES 0"
use std.strings.{RollingHash61}
let h = RollingHash61("")
print(yes(h.is_empty), h.len)

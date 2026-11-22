// expect: ok, output "YES"
use std.strings.{RollingHash, HashVal}
let h = RollingHash("abc")
print(yes(h.combine(h.all, HashVal(0, 0), 0) == h.all))

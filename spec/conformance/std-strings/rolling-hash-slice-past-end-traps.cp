// expect: R0401 at 3 (checked)
use std.strings.{RollingHash}
let h = RollingHash("ab")
print(h.slice(0, 3).h1)

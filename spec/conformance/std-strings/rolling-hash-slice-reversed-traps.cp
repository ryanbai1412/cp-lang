// expect: R0401 at 3 (checked)
use std.strings.{RollingHash}
let h = RollingHash("abab")
print(h.slice(2, 1).h1)

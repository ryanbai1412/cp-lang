// expect: ok, output "YES YES"
use std.strings.{HashVal}
let h = RollingHash("abab")
print(yes(h.slice(1, 1) == HashVal(0, 0)), yes(h.slice(0, 2) == h.slice(2, 4)))

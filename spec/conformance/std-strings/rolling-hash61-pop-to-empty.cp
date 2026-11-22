// expect: ok, output "YES"
use std.strings.{RollingHash61}
var h = RollingHash61("a")
h.pop()
print(yes(h.is_empty))

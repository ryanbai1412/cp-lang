// expect: ok, output "YES"
use std.strings.{RollingHash61}
var h = RollingHash61("")
h.push('a')
print(yes(h.all == RollingHash61("a").all))

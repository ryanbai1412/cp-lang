// expect: R0401 at 3 (checked)
use std.window.{Window}
var w = Window(+)
w.pop()
print(w.len)

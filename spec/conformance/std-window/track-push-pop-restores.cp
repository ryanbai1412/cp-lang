// expect: ok, output "4"
use std.window.{track}
var t = track(0, (mut s, x) => s += x, (mut s, x) => s -= x)
t.push(3)
t.push(4)
t.pop(3)
print(t.state)

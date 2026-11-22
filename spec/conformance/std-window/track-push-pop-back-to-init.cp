// expect: ok, output "0"
use std.window.{track}
var t = track(0, (mut s, x) => s += x, (mut s, x) => s -= x)
t.push(3)
t.pop(3)
print(t.state)

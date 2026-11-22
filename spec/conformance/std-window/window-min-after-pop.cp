// expect: ok, output "2"
use std.window.{Window}
var w = Window(min)
w.push(1)
w.push(2)
w.pop()
print(w.all!)

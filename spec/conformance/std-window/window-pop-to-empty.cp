// expect: ok, output "YES 0"
use std.window.{Window}
var w = Window(+)
w.push(2)
w.pop()
print(yes(w.all == None), w.len)

// expect: ok, output "5 2"
use std.window.{Window}
fn add<S: Track<Int>>(mut s: S, x: Int) = s.push(x)
var w = Window<Int, (Int, Int) -> Int>.new((a: Int, b: Int) => a + b)
add(mut w, 2)
add(mut w, 3)
print(w.all!, w.len)

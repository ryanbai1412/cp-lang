// expect: ok, output "0=5, 2=4"
use std.seq.{MonoStack}
var s = MonoStack<Int, Int>()
s.push(0, 5)
s.push(1, 3)
s.push(2, 4)
print(s)

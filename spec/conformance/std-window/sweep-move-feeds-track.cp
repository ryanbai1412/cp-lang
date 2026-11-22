// expect: ok, output "YES\nYES 3\n6 3\nYES 9 3\nYES 1"
use std.window.{Sweep}
domain D = 0..5
struct Sum(total: Int, n: Int):
    fn push(mut self, x: D):
        total += Int(x)
        n += 1
    fn pop(mut self, x: D):
        total -= Int(x)
        n -= 1
let s0 = Sweep<D, Sum>(Sum(0, 0))
print(s0.range.len == 0)
let s1 = Sweep<D, Sum>(Sum(0, 0), start: Some(D(1)..D(3)))
print(s1.range == D(1)..D(3), s1.state.total)
var s = Sweep<D, Sum>(Sum(0, 0))
s.move(D(1)..D(4))
print(s.state.total, s.state.n)
s.move(D(2)..=D(4))
print(s.range == D(2)..=D(4), s.state.total, s.state.n)
s.move(D(0)..D(2))
print(s.range.len == 2, s.state.total)

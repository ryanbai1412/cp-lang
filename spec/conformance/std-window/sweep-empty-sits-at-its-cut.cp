// expect: ok, output "YES 0\nYES 3\nYES 0\nYES 2\nYES"
domain D = 3..8
struct Sum(total: Int, n: Int):
    fn push(mut self, x: D):
        total += Int(x)
        n += 1
    fn pop(mut self, x: D):
        total -= Int(x)
        n -= 1
var s = Sweep<D, Sum>(Sum(0, 0), start: Some(D(5)..D(5)))
print(yes(s.range == D(5)..D(5)), s.state.n)
s.move(D(4)..D(7))
print(yes(s.range == D(4)..D(7)), s.state.n)
s.move(D(7)..D(7))
print(yes(s.range == D(7)..D(7)), s.state.n)
s.move(D(6)..=D(7))
print(yes(s.range == D(6)..=D(7)), s.state.n)
print(yes(Sweep<D, Sum>(Sum(0, 0)).range == D(3)..D(3)))

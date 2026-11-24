// expect: ok, output "99\n-4\n7"
// rule: 02.2.1
fn g(self: Int, i: 0..) -> Int = self + i
print(g(-1, 100))
print((-5).g(1))
let x = 3
print(x.g(4))

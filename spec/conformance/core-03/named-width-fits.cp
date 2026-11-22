// expect: ok, output "9"
// rule: 03.1.7
fn nine(k: Int) -> Int = if k == 0: 9 else: nine(k - 1)
struct Cell<T>:
    var v: T
var c = Cell<0..=9>(0)
c.v = nine(3)
print(c.v)

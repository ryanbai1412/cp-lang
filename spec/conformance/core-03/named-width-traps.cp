// expect: R0201 at 4
// rule: 03.1.7
fn ten(k: Int) -> Int = if k == 0: 10 else: ten(k - 1)
struct Cell<T>: var v: T
var c = Cell<0..=9>(0)
c.v = ten(3)

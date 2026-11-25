// expect: E0608
// rule: R05.2.2
struct B: xs: List<Int>
fn id<T>(x: T) -> T = x
let b = B([1, 2])
let c = id(b)
print(c.xs[1])
print(b.xs[0]) // <- here

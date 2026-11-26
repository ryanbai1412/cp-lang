// expect: ok, output "2\n1"
// rule: R05.5.3
// A generic fn's result that holds its argument lends it, so `b` is still readable.
struct B: xs: List<Int>
fn id<T>(x: T) -> T = x
let b = B([1, 2])
let c = id(b)
print(c.xs[1])
print(b.xs[0])

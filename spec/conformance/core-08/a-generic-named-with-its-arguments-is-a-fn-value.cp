// expect: ok, output "3\n3"
// rule: 08.2.5
// A generic fn or struct named as a value with its type arguments is the closure that calls it.
fn id<T>(x: T) -> T = x
struct Pair<T>:
    x: T
    y: T
let g = id<Int>
let mk = Pair<Int>
let p = mk(1, 2)
print(g(3))
print(p.x + p.y)

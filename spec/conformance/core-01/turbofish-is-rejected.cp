// expect: E0503
// rule: 01.3.4
// has: "::"
// Type arguments follow the name directly: one spelling, `id<Int>`.
fn id<T>(x: T) -> T = x
let g = id::<Int> // <- here
print(g(3))

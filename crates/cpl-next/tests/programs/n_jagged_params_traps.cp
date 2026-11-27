// expect: R0201
// rule: R06.4.1
// A parameter whose range ends at an earlier parameter is checked against that parameter's
// value as the body starts, at the parameter.
fn f(n: 0..=30, k: 0..=n) -> Int = n - k  // <- here
let a = 3
print(f(a, 3))
print(f(a, a + 1))

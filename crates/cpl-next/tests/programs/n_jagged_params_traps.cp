// expect: R0201
// rule: R06.4.1
// A parameter whose range ends at an earlier parameter is checked against that parameter's
// argument at the call, as an ordinary range parameter is.
fn f(n: 0..=30, k: 0..=n) -> Int = n - k
let a = 3
print(f(a, 3))
print(f(a, a + 1))  // <- here

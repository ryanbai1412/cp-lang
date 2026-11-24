// expect: E0702
// rule: 02.11.3
fn f(x: Int) -> Int = x
fn f(x: Str) -> Int = 0 // <- here

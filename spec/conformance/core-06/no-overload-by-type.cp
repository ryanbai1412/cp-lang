// expect: E0702
// rule: 06.1.6
fn f(x: Int) -> Int = x
fn f(x: Str) -> Int = 0 // <- here

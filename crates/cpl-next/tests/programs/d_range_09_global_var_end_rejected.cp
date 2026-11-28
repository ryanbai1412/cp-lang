// expect: E0326
// rule: R02.10.3
// A top-level `var` may change while a value of the type is alive, so it can't end one.
var m: 1..=10000 = 3
fn keep(c: 1..=m) -> Int = c // <- here
print(keep(3))

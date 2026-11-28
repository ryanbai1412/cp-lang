// expect: E0326
// engines: next
// rule: 02.10.3
// A top-level `var` can't end a type: it may change while the type is in use.
var m = 5
fn twice(c: 1..=m) -> Int = c * 2 // <- here
print(twice(3))

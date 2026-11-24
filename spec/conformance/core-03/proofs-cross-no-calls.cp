// expect: R0201
// rule: 03.4.2
fn twelve() -> Int = 12
var x: 0..=9 = 0
x = twelve() // <- here
print(x)

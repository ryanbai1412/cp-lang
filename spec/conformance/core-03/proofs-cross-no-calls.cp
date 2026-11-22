// expect: R0201 at 3
// rule: 03.4.2
fn twelve() -> Int = 12
var x: 0..=9 = 0
x = twelve()
print(x)

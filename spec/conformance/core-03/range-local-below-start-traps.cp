// expect: R0201
// rule: 03.1.4
var x: -5..=5 = -5
let step = 1
x -= step // <- here
print(x)

// expect: R0201
// rule: 03.1.4
var x: 1..=3 = 3
let step = 1
x += step // <- here
print(x)

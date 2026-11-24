// expect: E0103
// rule: 01.6.4
// Compound assignment is a statement, not a value.
var x = 1
let y = (x += 2) // <- here
print(y)

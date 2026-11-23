// expect: E0103 at 3
// rule: 01.6.4
// Compound assignment is a statement, not a value.
var x = 1
let y = (x += 2)
print(y)

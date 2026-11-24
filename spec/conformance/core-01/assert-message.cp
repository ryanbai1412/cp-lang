// expect: R0401
// rule: 01.6.6
let x = 3
assert x > 0
assert x > 5, "x is {x}" // <- here

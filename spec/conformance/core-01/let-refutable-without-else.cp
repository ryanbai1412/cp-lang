// expect: E0106
// rule: 01.6.2
let p = (1, 2)
let (1, b) = p // <- here
print(b)

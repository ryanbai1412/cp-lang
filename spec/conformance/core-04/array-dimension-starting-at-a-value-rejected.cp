// expect: E0302
// rule: 04.3.4
let n = 2
let a: [n..4]Int = [1, 2] // <- here
print(a.len)

// expect: E0301
// rule: 04a.7.1
domain R = 0..2
domain C = 0..3
domain P = (R, C)
let g: [R, C]Int = [P] p => 1
let row = g[R(1)]
let r = R(0)
print(row[r]) // <- here

// expect: R0102
// rule: 04.2.10
domain R = 0..3
domain C = 0..3
domain P = (R, C)
let i = 3
let q = P((i, 0)) // <- here

// expect: ok, output "5 5 10"
// rule: 04.2.11
// `t[(r, c)]` and `t[r, c]` are one access with one type.
domain Row = 0..2
domain Col = 0..3
var t: [Row, Col]Int = 0
t[Row.first!, Col.first!] = 5
let a: Int = t[(Row.first!, Col.first!)]
let b = t[Row.first!, Col.first!]
let same = [a, b]
print(a, b, same.sum())

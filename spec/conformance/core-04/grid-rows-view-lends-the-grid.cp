// expect: E0602
// rule: 04b.1.12 05.7.1
// A bound `g.rows` lends `g` while it lives, like any `lends` view (adversarial round 47).
domain R = 0..2
domain C = 0..2
var g: [R, C]Int = [R, C] (r, c) => 0
let rows = g.rows
g[R(0), C(0)] = 1 // <- here
print(rows.len)

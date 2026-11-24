// expect: E0602 at 6
// rule: 04b.1.12 05.7.1
// A row taken from `g.rows` lends `g` while it lives (adversarial round 47).
domain R = 0..2
domain C = 0..2
var g: [R, C]Int = [R, C] (r, c) => 0
let r0 = g.rows[R(0)]
g[R(0), C(0)] = 1
print(r0.len)

// expect: E0602
// rule: 04b.1.12 05.7.1
// A `for` over `g.rows` lends `g` until the loop ends (adversarial round 47).
domain R = 0..2
domain C = 0..2
var g: [R, C]Int = [R, C] (r, c) => 0
for row in g.rows:
    g[R(0), C(0)] += row.len // <- here
print(g[R(0), C(0)])

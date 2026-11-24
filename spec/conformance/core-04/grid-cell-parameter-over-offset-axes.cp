// expect: ok, output "23 5"
// rule: 04.2.11
// One table name over two axes binds the cell tuple when the axes start past 0, and its parts index arrays over each axis.
domain R = 1..=2
domain C = 3..=4
let w: [R]Int = [R] r => Int(r) * 10
let t = [R, C] p => w[p.0] + Int(p.1)
print(t[R(2), C(3)], [p for p in t.keys].len + 1)

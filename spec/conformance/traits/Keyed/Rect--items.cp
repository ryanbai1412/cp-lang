// expect: ok, output "0 1 1\n0 2 2\n1 1 4\n1 2 5"
// engines: next
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 3 + Int(c)
let q = g[R(0)..=R(1), C(1)..=C(2)]
for ((r, c), x) in q.items(): print(r, c, x)

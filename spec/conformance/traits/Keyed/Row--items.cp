// expect: ok, output "0 3\n1 4\n2 5"
// engines: next
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 3 + Int(c)
let w = g[R(1)]
for (c, x) in w.items(): print(c, x)

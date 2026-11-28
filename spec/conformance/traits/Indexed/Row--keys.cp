// expect: ok, output "0\n1\n2"
// engines: next
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 3 + Int(c)
let w = g[R(1)]
for c in w.keys: print(c)

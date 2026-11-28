// expect: ok, output "0 1\n1 4"
// engines: next
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 3 + Int(c)
let k = g[.., C(1)]
for (r, x) in k.items(): print(r, x)

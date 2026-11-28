// expect: ok, output "4\n1"
// engines: next
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 3 + Int(c)
let k = g[.., C(1)]
for x in k.iter_rev(): print(x)

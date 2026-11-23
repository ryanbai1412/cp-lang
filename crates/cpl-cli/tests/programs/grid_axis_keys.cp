domain R = 1..=2
domain C = 1..=3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 10 + Int(c)
let part = g[R(2), C(2)..]
print(part.keys.first!, part[C(3)])
for (c, x) in g[R(1)].items: print(c, x)
print(g[.., C(3)].keys.last!, g[R(2)].len)

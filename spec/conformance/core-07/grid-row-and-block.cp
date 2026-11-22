// expect: ok, output "66 70"
// rule: 07.4.12 07.4.13
domain R = 1..=2
domain C = 1..=3
let g = [R, C] (r, c) => Int(r) * 10 + Int(c)
print(g[R(2)].sum(), g[R(1)..=R(2), C(2)..=C(3)].sum())

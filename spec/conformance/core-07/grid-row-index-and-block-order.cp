// expect: ok, output "23 12 13 22 23"
// rule: 07.4.12 07.4.13
domain R = 1..=2
domain C = 1..=3
let g = [R, C] (r, c) => Int(r) * 10 + Int(c)
let row2 = g[R(2)]
let blk = g[R(1)..=R(2), C(2)..=C(3)]
let xs = [x for x in blk]
print(row2[C(3)], xs)

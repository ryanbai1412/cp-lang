// cp: 0.1 bea787c+dirty
domain R = 0..2
domain C = 0..2
let g: [R, C]Int = read()
let pre = prefix(g)
print(pre.sum(R(0)..=R(1), C(0)..=C(1)))

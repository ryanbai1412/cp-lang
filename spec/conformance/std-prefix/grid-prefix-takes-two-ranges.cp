// expect: E0303 at 5
domain R = 0..2
domain C = 0..3
let a: [R, C]Int = [R, C] (r, c) => 1 + 3 * Int(r) + Int(c)
let p = Prefix(a)
print(p.fold(0..=1))

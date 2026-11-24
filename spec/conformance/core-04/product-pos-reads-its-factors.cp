// expect: ok, output "1 2\n1 2"
// rule: 04.2.10
// A product position viewed as its factor tuple: `.0`/`.1` and a tuple pattern read its factors.
domain R = 0..3
domain C = 0..3
domain Cell = (R, C)
let p = Cell((R(1), C(2)))
print(Int(p.0), Int(p.1))
let (r, c) = p
print(Int(r), Int(c))

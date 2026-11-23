// An inline product `(R, C)` is a domain value, a `Seq` of its cells
// row-major, like a named `domain P = (R, C)` (R04.2.20).
enum Dir = N | E | S | W
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 3 + Int(c)
print((R, C).count(p => g[p] % 2 == 0), (R, C).len)
print((R, Dir).count(p => p.1 != Dir.N))
let d = (R, C)
print(d.len)

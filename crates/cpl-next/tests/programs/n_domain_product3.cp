// expect: ok, output "5 0 12\n11 23\n4 4 23\n16 15"
// rule: R04.2.9 R04.2.10 R02.5
// Products of three and four domains are domains through std.tuple's ordinary impls, walked
// row-major with the last factor fastest, like a pair.
domain A = 0..2
domain B = 0..3
domain C = 0..2
var g: [A, B, C]Int = 0
g[A(1), B(2), C(1)] = 5
print(g[(A(1), B(2), C(1))], g[A(0), B(0), C(0)], g.len)
let p = (A(1), B(2), C(1))
print(p.ord, (A, B, C).nth(11).ord + 12)
type Cell = (A, B, C)
let q = Cell((1, 1, 1))
print(q.ord / 2, Int(q._1) + 3, Cell.count * 2 - 1)
type Quad = (A, A, A, A)
var n = 0
for v in Quad: n += 1
print(n, Quad((1, 1, 1, 1)).ord)

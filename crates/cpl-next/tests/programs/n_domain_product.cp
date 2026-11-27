// expect: ok, output "1 2 6\n0 0\n7 7\n36\n4"
// rule: R04.2.9 R04.2.10 R04.2.11
// A product domain is the tuple of its factors: `[Cell]T` is `[R, C]T`, `(R, C)` walks row-major,
// a position destructures, and `g[r, c]` is `g[(r, c)]`.
domain R = 0..2
domain C = 0..3
domain Cell = (R, C)
var g: [R, C]Int = 0
g[R(1), C(2)] = 7
let p = Cell.find(q => g[q] == 7)!
print(Int(p._0), Int(p._1), Cell.count)
let (r, c) = Cell.first!
print(Int(r), Int(c))
let h: [Cell]Int = g
print(h[(R(1), C(2))], h[R(1), C(2)])
let t = [R, C] q => Int(q._0) * 10 + Int(q._1)
var s = 0
for (a, b) in (R, C):
    s += t[a, b]
print(s)
print((R(1), C(1)).ord)

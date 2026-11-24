// expect: ok, output "5 2 3\n5 2\n3 5"
// rule: 06.1.5
// A bare operator in a tuple or record keeps its identity: `each(+, max, min)` and the named
// `each(s: +, h: max)` are field-wise monoids whose `Monoid.of` sees `+`, `max` and `min`.
let m = Monoid<(Int, Int, Int)>.of(each(+, max, min))
print(m.op((1, 2, 3), (4, 0, 9)))
let n = Monoid<(s: Int, h: Int)>.of(each(s: +, h: max))
let v = n.op((s: 1, h: 2), (s: 4, h: 0))
print(v.s, v.h)
print(fold(each(+, max), [(1, 5), (2, 3)]))

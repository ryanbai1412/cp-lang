// expect: ok, output "20 1"
// engines: next
let m = Monoid<Int>(1, (a, b) => a * b).commutative()
print(m.monoid().op(4, 5), m.monoid().id)

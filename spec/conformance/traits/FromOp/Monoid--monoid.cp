// expect: ok, output "5 0"
// engines: next
let m = Monoid<Int>(0, (a, b) => a + b)
print(m.monoid().op(2, 3), m.monoid().id)

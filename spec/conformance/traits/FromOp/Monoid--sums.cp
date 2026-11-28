// expect: ok, output "NO 0"
// engines: next
let m = Monoid<Int>(0, (a, b) => a + b)
print(yes(m.sums()), m.monoid().id)

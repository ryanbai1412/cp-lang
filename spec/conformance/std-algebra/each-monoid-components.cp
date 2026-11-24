// expect: ok, output "0 -100 3 5 2 10"
// rule: 06.1.5
// `each` over named `Monoid` components is the field-wise monoid on records; named variadic
// args make a record (decision 324), `fields` walks the ops and both operands together, and
// `T(...)` builds the record back. Bare operators inside `each` still need the compiler (stdlib PLAN).
let add = Monoid<Int>(0, (a, b) => a + b)
let hi = Monoid<Int>(-100, (a, b) => max(a, b))
let m = Monoid<(n: Int, h: Int)>.of(each(n: add, h: hi))
let c = m.op((n: 1, h: 5), (n: 2, h: 3))
fn dbl<T>(a: T) -> T = T(x * 2 for x in fields(a))
let (x, y) = dbl((1, 5))
print(m.id.n, m.id.h, c.n, c.h, x, y)

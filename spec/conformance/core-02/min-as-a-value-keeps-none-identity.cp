// expect: ok, output "3 3 3\n3 3 -1\n1 7 0.5000000000 5 2"
// rule: 02.4.6
// `min` and `max` named as values are the fns a direct call is: `None` is still the identity.
let a: ?Int = None
let b: ?Int = Some(3)
print(min(a, b) ?? -1, Monoid<?Int>(None, min).op(a, b) ?? -1, Monoid<?Int>(None, (x, y) => min(x, y)).op(a, b) ?? -1)
let m = Monoid<?Int>(None, max)
print(m.op(a, b) ?? -1, m.op(b, a) ?? -1, m.op(a, a) ?? -1)
let f = min
print([4, 1, 7].fold(min)!, [4, 1, 7].fold(max)!, [2.5, 0.5].fold(min)!, Monoid<Int>(0, max).op(2, 5), f(3, 2))

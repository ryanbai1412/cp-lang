// expect: ok, output "1180591620717411303424 1180591620717411303424"
// rule: 08.1.5
// `**` on an `I128` is its `Pow` impl and runs past `Int`'s range, directly and through a bound.
fn sq<T: Pow>(x: T) -> T = x ** 2
print(I128(2) ** 70, sq(I128(2) ** 35))

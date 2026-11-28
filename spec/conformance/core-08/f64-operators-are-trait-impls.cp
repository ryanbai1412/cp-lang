// expect: ok, output "0.7500000000 3.0000000000 -2.5000000000 0.5000000000 2.0000000000"
// rule: 08.1.5
// F64's operators are std trait impls, so generic code over them takes an F64.
fn twice<T: Add>(x: T) -> T = x + x
fn flip<T: Neg>(x: T) -> T = -x
fn less<T: Sub>(a: T, b: T) -> T = a - b
fn ratio<T: Mul + Div>(a: T, b: T) -> T = a * a / b
print([0.5, 0.25].sum(), twice(1.5), flip(2.5), less(1.0, 0.5), ratio(1.0, 0.5))

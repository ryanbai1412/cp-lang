use std.algebra.{desc}
fn ap<T, K>(x: T, by: T -> K) -> K = by(x)
fn wrap<T>(x: T) -> List<T> = [x]
domain V = 0..3
print(Int(ap(V.last!, desc).x))
print(ap("ab", desc).x, ap("c", wrap).len)
print(yes(ap(V.first!, desc) < ap(V.last!, desc)))

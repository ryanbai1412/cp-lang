// expect: ok, output "-9223372036854775807 9223372036854775807 -10 10 3 7 0 127 0 1 3"
// rule: R02.2.1
// `T.MIN`/`T.MAX` through `T: Bounded`: `Int`'s, a range type's own ends (an instance keeps the
// range it is given, decision 580), `Char`'s and `Bool`'s.
type M = -10..=10
fn lo<T: Bounded>() -> T = T.MIN
fn hi<T: Bounded>() -> T = T.MAX
fn first<T: Domain>() -> T = T.nth(0)
let b = if hi<Bool>(): 1 else: 0
print(Int.MIN, Int.MAX, lo<M>(), hi<M>(), lo<3..=7>(), hi<3..=7>(), Int(lo<Char>()), Int(hi<Char>()), Int(lo<Bool>()), b, Int(first<3..=7>()))

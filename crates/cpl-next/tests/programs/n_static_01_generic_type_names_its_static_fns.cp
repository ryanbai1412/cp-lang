// expect: ok, output "0\n7\n3"
// A generic type written with its arguments names its trait's fns that take no `self`, as a
// plain type name does: `W<Int>.zero()` is `W<Int>`'s impl of `Zero`.
trait Zero:
    fn zero() -> Self
    fn seven() -> Int
struct W<T>(x: Int)
impl Zero for W<T>:
    fn zero() -> W<T> = W(0)
    fn seven() -> Int = 7
print(W<Int>.zero().x)
print(W<Bool>.seven())
print(W<Int>.zero().x + 3)

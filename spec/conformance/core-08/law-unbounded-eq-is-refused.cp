// expect: E0606
// rule: R08.3.6
// engines: next
// A law's body is checked like a trait default, once for every `T`: `==` on a `T` with no `Eq` is refused.
trait Op<T>:
    fn op(self, a: T, b: T) -> T
trait Selective<T>: Op<T>:
    law select(a: T, b: T) = self.op(a, b) == a // <- here
print(1)

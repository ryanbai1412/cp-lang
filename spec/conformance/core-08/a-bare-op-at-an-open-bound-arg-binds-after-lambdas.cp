// expect: ok, output "3"
// rule: 08.1.9
// A bare op given for `M: Combine<U>` while `U` is still open binds after the call's lambdas, with the impls its bound picks; an open `U` takes `Int` from the op, as a literal does (R08.2.11; G7.3).
trait Combine<T>:
    fn combine(self, a: T, b: T) -> T
trait HasId<T>:
    fn id(self) -> T
impl<T: Add> Combine<T> for type_of(+):
    fn combine(self, a: T, b: T) -> T = a + b
impl<T: Zero> HasId<T> for type_of(+):
    fn id(self) -> T = T.zero
fn up<T, U: One, M: Combine<U>>(op: M, finish: U -> T) -> T:
    if M: HasId<U>:
        return finish(op.id())
    return finish(U.one)
print(up(+, s => s + 3))

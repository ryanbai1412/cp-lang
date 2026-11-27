// expect: ok, output "5 6 4 0 24"
// rule: R08.1.9 R08.1.10
// A bare operator is its `@operator` trait's fn item. Passed at a bound it has no impl of, it and a
// fn item become the call of the `@from_op` fn of a struct that has one, when they meet that fn's
// bound, as `big` with its own `Lift` impl is.
struct Mon<T>(id: T, op: (T, T) -> T):
    @from_op
    pub fn of<M: Lift<T>>(m: M) -> Mon<T> = m.lift()
trait Lift<T>:
    fn lift(self) -> Mon<T>
impl<T: Add + Zero> Lift<T> for type_of(+):
    fn lift(self) -> Mon<T> = Mon(T.zero, (a, b) => a + b)
impl Lift<Int> for type_of(*):
    fn lift(self) -> Mon<Int> = Mon(1, (a, b) => a * b)
fn big(a: Int, b: Int) -> Int = if a > b: a else: b
impl Lift<Int> for type_of(big):
    fn lift(self) -> Mon<Int> = Mon(-1000, big)
trait Fold3<T>:
    type Out
    fn fold2(self, a: T, b: T) -> Self.Out
impl<T> Fold3<T> for Mon<T>:
    type Out = T
    fn fold2(self, a: T, b: T) -> T = self.op(self.op(self.id, a), b)
impl<T> Fold3<T> for (T, T) -> T:
    type Out = ?T
    fn fold2(self, a: T, b: T) -> ?T = Some(self(a, b))
fn go<M: Fold3<E>, E>(m: M, a: List<E>) -> M.Out = m.fold2(a[0], a[1])
let x: Int = go(+, [2, 3])
let y: Int = go(*, [2, 3])
let z: Int = go(big, [4, 1])
print(x, y, z, Mon<Int>.of(+).id, Mon<Int>.of(*).op(4, 6))

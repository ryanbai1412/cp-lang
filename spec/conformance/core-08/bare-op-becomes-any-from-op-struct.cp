// expect: ok, output "5 5"
// rule: 08.1.9
// A bare operator passed where a struct with a `@from_op` static fn is wanted becomes that fn's call, a user's struct as much as std's `Monoid`; at a trait bound it keeps its own type instead (R08.1.9).
struct Sum<T: Add + Zero>(id: T, op: (T, T) -> T):
    @from_op
    pub fn of<M>(m: M) -> Sum<T> = Sum(T.zero, (a, b) => a + b)
trait Op3<T>:
    fn fold2(self, a: T, b: T) -> T
impl Op3<Int> for type_of(+):
    fn fold2(self, a: Int, b: Int) -> Int = a + b
fn go(s: Sum<Int>, a: List<Int>) -> Int = s.op(s.op(s.id, a[0]), a[1])
fn at<M: Op3<E>, E>(m: M, a: List<E>) -> E = m.fold2(a[0], a[1])
print(go(+, [2, 3]), at(+, [2, 3]))

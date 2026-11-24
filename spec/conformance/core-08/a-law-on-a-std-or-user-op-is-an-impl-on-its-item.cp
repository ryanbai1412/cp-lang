// expect: ok, output "1 0 1 1 0\nYES 4 4 0 YES"
// rule: std.algebra.Selective 08.1.9
// A law is an impl on the op's own type: `max`, `min` and a user `mx` are `Selective`, `+` and a closure are not, and `Monoid.of` finds `mx`'s identity by impl as it finds std's.
use std.algebra.{HasId}
fn mx(a: Int, b: Int) -> Int = if a > b: a else: b
impl Op<Int> for type_of(mx):
    type Fold = Int
    fn op(self, a: Int, b: Int) -> Int = mx(a, b)
    fn folds(self) -> Monoid<Int> = Monoid<Int>.of(self)
    fn lift(self, x: Int) -> Int = x
impl HasId<Int> for type_of(mx):
    @get fn id(self) -> Int = Int.MIN
impl Idempotent<Int> + Selective<Int> for type_of(mx)
fn pick<T, M: Op<T>>(m: M, a: T, b: T) -> Int:
    if M: Selective<T>: return 1
    0
fn total<T, M: Op<T>>(m: M, xs: List<T>) -> M.Fold:
    let f = m.folds()
    var acc = f.id
    for x in xs: acc = f.op(acc, m.lift(x))
    acc
print(pick(max, 1, 2), pick(+, 1, 2), pick(mx, 1, 2), pick(min, 1, 2), pick((a: Int, b: Int) => a + b, 1, 2))
print(yes(Monoid<Int>.of(mx).id == Int.MIN), total(mx, [3, 4, 1]), total(max, [3, 4, 1]), Monoid<Int>.of(+).id, yes(Monoid<Int>.of(max).id == Int.MIN))

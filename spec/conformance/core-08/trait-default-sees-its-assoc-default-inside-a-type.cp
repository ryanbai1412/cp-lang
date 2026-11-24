// expect: ok, output "5 -1"
// rule: 08.1.9
// A default body sees `Self.Fold` as the trait's default `?T` wherever the signature names it, here inside `Monoid<Self.Fold>` (decision 469).
trait Op8<T>:
    type Fold = ?T
    fn op(self, a: T, b: T) -> T
    fn folds(self) -> Monoid<Self.Fold> =
        Monoid(None, (a: ?T, b: ?T) => if let x = a: (if let y = b: Some(self.op(x, y)) else: a) else: b)
struct Add
impl Op8<Int> for Add:
    fn op(self, a: Int, b: Int) -> Int = a + b
let m = Add().folds()
print(m.op(Some(2), Some(3)) ?? -1, m.id ?? -1)

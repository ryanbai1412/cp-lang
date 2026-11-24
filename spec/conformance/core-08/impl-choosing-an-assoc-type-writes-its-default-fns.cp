// expect: E0606
// rule: 08.1.9
// An impl that chooses an associated type the trait gives a default must write each default fn whose signature names it: those bodies were written for the default (decision 469).
trait Wrap<T>:
    type Out = ?T
    fn base(self, x: T) -> T
    fn wrap(self, x: T) -> Self.Out = Some(self.base(x))
struct Inc
impl Wrap<Int> for Inc:
    type Out = Int   // <- here
    fn base(self, x: Int) -> Int = x + 1
print(Inc().base(1))

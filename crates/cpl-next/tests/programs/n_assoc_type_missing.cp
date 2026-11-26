// expect: E0606
// rule: R08.1.9
// An impl that leaves out an associated type with no default is refused, naming it.
trait Src:
    type Item
    fn get(self) -> Self.Item
struct A(x: Int)
impl Src for A: // <- here
    fn get(self) -> Int = self.x
print(A(3).get())

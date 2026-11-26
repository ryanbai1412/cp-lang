// expect: E0606
// rule: R08.1.1
// An impl's choice for an associated type must have the traits the trait's bound names.
trait Label:
    fn label(self) -> Int
trait Src:
    type Item: Label
    fn get(self) -> Self.Item
struct A(x: Int)
impl Src for A:
    type Item = Int // <- here
    fn get(self) -> Int = self.x
print(A(3).get())

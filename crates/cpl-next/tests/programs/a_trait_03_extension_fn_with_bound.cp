// expect: ok, output "5"
// rule: R08.1.2, R08.1.4
trait Get:
    fn get(self) -> Int
struct B: x: Int
impl Get for B:
    fn get(self) -> Int = self.x
fn plus<S: Get>(self: S, k: Int) -> Int = self.get() + k
print(B(4).plus(1))

// expect: ok, output "6"
// rule: R08.1.2
// An impl may be for a type that is not a struct.
trait Twice:
    fn twice(self) -> Int
impl Twice for Int:
    fn twice(self) -> Int = self * 2
fn go<T: Twice>(x: T) -> Int = x.twice()
print(go(3))

// expect: E0606
// rule: R08.1.4
// A trait fn's `where Self: Big`, called through a bound, asks the caller's parameter for `Big`.
trait Big:
    fn size() -> Int
trait Thing:
    fn id(self) -> Int
    fn twice(self) -> Int where Self: Big = self.id() + Self.size()
struct B:
    x: Int
impl Thing for B:
    fn id(self) -> Int = x
fn h<T: Thing>(t: T) -> Int = t.twice()
print(h(B(1)))

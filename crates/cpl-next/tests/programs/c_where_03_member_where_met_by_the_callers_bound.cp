// expect: ok, output "11"
// rule: R08.1.4
// A trait fn's `where Self: Big` is met by the caller's own bound.
trait Big:
    fn size() -> Int
trait Thing:
    fn id(self) -> Int
    fn twice(self) -> Int where Self: Big = self.id() + Self.size()
struct B:
    x: Int
impl Thing for B:
    fn id(self) -> Int = x
fn h<T: Thing + Big>(t: T) -> Int = t.twice()
impl Big for B:
    fn size() -> Int = 10
print(h(B(1)))

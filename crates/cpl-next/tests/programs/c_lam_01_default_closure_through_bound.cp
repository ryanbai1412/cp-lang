// expect: ok, output "6 8 10"
// rule: R08.1.4
// A trait fn's defaulted closure parameter is filled the same through a concrete receiver and through a bound.
trait Bag:
    fn get(self) -> Int
    fn twice(self, f: Int -> Int = x => x * 2) -> Int = f(self.get())
struct B:
    v: Int
impl Bag for B:
    fn get(self) -> Int = self.v
fn go<T: Bag>(t: T) -> Int = t.twice()
print(B(3).twice(), go(B(4)), go(B(5)))

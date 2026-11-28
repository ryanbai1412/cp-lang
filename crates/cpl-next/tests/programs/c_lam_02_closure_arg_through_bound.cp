// expect: ok, output "4 8"
// rule: R08.1.4
// A lambda passed to a trait fn through a bound is put in the set the call passes the impl.
trait Bag:
    fn get(self) -> Int
    fn twice(self, f: Int -> Int) -> Int = f(self.get())
struct B:
    v: Int
impl Bag for B:
    fn get(self) -> Int = self.v
fn go<T: Bag>(t: T) -> Int = t.twice(x => x * 2)
print(B(3).twice(x => x + 1), go(B(4)))

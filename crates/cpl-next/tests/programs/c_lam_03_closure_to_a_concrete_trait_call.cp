// expect: ok, output "4 23"
// rule: R08.1.4
// A lambda passed to a trait fn on a concrete receiver reaches the trait default or the impl's own fn.
trait Bag:
    fn get(self) -> Int
    fn twice(self, f: Int -> Int) -> Int = f(self.get())
struct B:
    v: Int
impl Bag for B:
    fn get(self) -> Int = self.v
struct C:
    v: Int
impl Bag for C:
    fn get(self) -> Int = self.v
    fn twice(self, f: Int -> Int) -> Int = f(f(self.v))
let k = 10
print(B(3).twice(x => x + 1), C(3).twice(x => x + k))

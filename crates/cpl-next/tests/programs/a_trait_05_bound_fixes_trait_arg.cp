// expect: ok, output "7"
// rule: R08.1.2
trait Get<T>:
    fn get(self) -> T
struct B: x: Int
impl Get<Int> for B:
    fn get(self) -> Int = self.x
fn twice<S: Get<T>, T>(s: S) -> T = s.get()
let v = twice(B(7))
print(v + 0)

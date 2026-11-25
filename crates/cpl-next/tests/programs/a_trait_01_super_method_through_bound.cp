// expect: ok, output "4 5"
// rule: R08.1.1, R08.1.2
// A bound's supertrait gives its methods, with the trait's arguments put in.
trait Get<T>:
    fn get(self) -> T
trait Both<T>: Get<T>:
    fn other(self) -> T
struct Pair: a: Int, b: Int
impl Get<Int> for Pair:
    fn get(self) -> Int = self.a
impl Both<Int> for Pair:
    fn other(self) -> Int = self.b
fn show<P: Both<Int>>(p: P) = print(p.get(), p.other())
show(Pair(4, 5))

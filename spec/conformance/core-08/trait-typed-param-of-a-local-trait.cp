// expect: ok, output "30 12"
// rule: 08.2.1
// A free fn whose parameter is typed by a trait the file declares is generic in that argument's type, as one typed by a std trait such as `Seq` is.
trait Big<V>:
    fn big(self) -> V
struct A(x: Int)
struct B(s: Str)
impl Big<Int> for A:
    fn big(self) -> Int = self.x * 10
impl Big<Int> for B:
    fn big(self) -> Int = self.s.len * 4
fn show(b: Big<Int>) -> Int = b.big()
print(show(A(3)), show(B("abc")))

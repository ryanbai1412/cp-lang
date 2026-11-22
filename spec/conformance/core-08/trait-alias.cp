// expect: ok, output "sq 9"
// rule: 08.2.6 12.11.1
trait Sh:
    fn area(self) -> Int
trait Nm:
    fn name(self) -> Str
trait Both = Sh + Nm
struct Sq: s: Int
impl Sh for Sq:
    fn area(self) -> Int = self.s * self.s
impl Nm for Sq:
    fn name(self) -> Str = "sq"
fn show<T: Both>(x: T) = print(x.name(), x.area())
show(Sq(3))

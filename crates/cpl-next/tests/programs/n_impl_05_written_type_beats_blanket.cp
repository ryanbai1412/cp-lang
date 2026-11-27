// expect: ok, output "concrete blanket concrete blanket"
// rule: R08.1.2
// An impl for a written type beats a blanket impl over a bounded parameter; other types take the blanket.
trait Tagged:
    fn tag(self) -> Int
trait Name:
    fn name(self) -> Str
impl<T: Tagged> Name for T:
    fn name(self) -> Str = "blanket"
impl Name for Int:
    fn name(self) -> Str = "concrete"
impl Tagged for Int:
    fn tag(self) -> Int = 1
impl Tagged for Char:
    fn tag(self) -> Int = 2
fn show<X: Name>(x: X) -> Str = x.name()
print(5.name(), 'a'.name(), show(5), show('a'))

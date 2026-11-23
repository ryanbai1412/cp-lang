// expect: ok, output "hi a yo b"
// rule: 08.5.1
// A default method is used as is or overridden once, in the impl; there is no chain.
trait Named:
    fn name(self) -> Str
    fn greet(self) -> Str = "hi " + self.name()
struct A()
struct B()
impl Named for A:
    fn name(self) -> Str = "a"
impl Named for B:
    fn name(self) -> Str = "b"
    fn greet(self) -> Str = "yo b"
print(A().greet(), B().greet())

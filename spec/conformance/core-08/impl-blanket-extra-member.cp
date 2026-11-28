// expect: E0203
// rule: 08.1.2
// tool: check
// has: "`Scaled` has no fn `twice`"
// has: "a helper of the type's own goes in the type's own `impl`"
// A blanket impl's body holds only the trait's members too.
trait Base:
    fn value(self) -> Int
trait Scaled:
    fn scaled(self) -> Int
struct One(n: Int)
impl Base for One:
    fn value(self) -> Int = n
impl<T: Base> Scaled for T:
    fn scaled(self) -> Int = self.value() * 10
    fn twice(self) -> Int = self.value() * 2 // <- here
print(One(4).scaled())

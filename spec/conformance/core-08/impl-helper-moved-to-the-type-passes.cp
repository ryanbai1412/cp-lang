// expect: ok, output "6 3"
// rule: 08.1.2
// A helper that is no trait member goes in the type's own `impl`, beside the trait's.
trait Sized:
    fn total(self) -> Int = 0
struct Box3(a: Int, b: Int, c: Int)
impl Sized for Box3:
    fn total(self) -> Int = a + b + c
impl Box3:
    fn count(self) -> Int = 3
let b = Box3(1, 2, 3)
print(b.total(), b.count())

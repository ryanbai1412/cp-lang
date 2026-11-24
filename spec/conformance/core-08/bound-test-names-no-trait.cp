// expect: E0204
// rule: 08.2.9
// has: "no trait named `Selectve`"
// `if M: Tr<T>:` names a trait; a misspelt one is reported where it is written, not tested as false.
trait Op<T>:
    fn op(self, a: T, b: T) -> T
struct Pl
impl Op<Int> for Pl:
    fn op(self, a: Int, b: Int) -> Int = a + b
fn pick<T, M: Op<T>>(m: M, a: T, b: T) -> Int = if M: Selectve<T>: 1 else: 0  // <- here
print(pick(Pl(), 1, 2))

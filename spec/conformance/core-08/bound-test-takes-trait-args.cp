// expect: ok, output "1 0 1 2 0"
// rule: 08.2.9
// engines: next
// `if M: Tr<T>:` tests a bound with its trait args, per instance at compile time: the branch taken
// sees `M: Tr<T>` (`m.choose`), and an impl at other args (`Chooser<F64>` for `Lo`) doesn't count.
trait Op<T>:
    fn op(self, a: T, b: T) -> T
trait Selective<T>: Op<T>:
    law select(a: T, b: T) where T: Eq = self.op(a, b) == a or self.op(a, b) == b
trait Chooser<T>: Op<T>:
    fn choose(self, a: T, b: T) -> Bool
struct Mx
impl Op<Int> for Mx:
    fn op(self, a: Int, b: Int) -> Int = if a > b: a else: b
impl Selective<Int> for Mx
impl Chooser<Int> for Mx:
    fn choose(self, a: Int, b: Int) -> Bool = a > b
struct Lo
impl Op<Int> for Lo:
    fn op(self, a: Int, b: Int) -> Int = if a < b: a else: b
impl Chooser<F64> for Lo:
    fn choose(self, a: F64, b: F64) -> Bool = a < b
fn pick<T, M: Op<T>>(m: M, a: T, b: T) -> Int = if M: Selective<T>: 1 else: 0
fn first<T, M: Op<T>>(m: M, a: T, b: T) -> Int:
    if M: Chooser<T>:
        return if m.choose(a, b): 1 else: 2
    0
print(pick(Mx(), 1, 2), pick(Lo(), 1, 2), first(Mx(), 3, 2), first(Mx(), 1, 2), first(Lo(), 1, 2))

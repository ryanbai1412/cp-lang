// expect: ok, output "0 1 0"
// rule: 08.1.9
// An omitted `op: G = (+)` binds G to `type_of(+)` with the impl its bound picks, as a written `+` does: its impl's own bounds see the bound's `Int` (G7.3).
trait Id<T>:
    fn id(self) -> T
impl<T: Zero> Id<T> for type_of(+):
    fn id(self) -> T = T.zero
impl<T: One> Id<T> for type_of(*):
    fn id(self) -> T = T.one
fn g<G: Id<Int>>(op: G = (+)) -> Int = op.id()
print(g(), g(*), g(+))

// expect: ok, output "3 x"
// rule: R02.10.4
// An impl's type parameters declared in `<..>` are its own whatever their names: `Elem`
// and `Key` with no bound are parameters, not types to look up.
trait Pair<A, B>:
    fn left(self) -> A
    fn right(self) -> B
struct Two<E, K>(e: E, k: K)
impl<Elem, Key> Pair<Elem, Key> for Two<Elem, Key>:
    fn left(self) -> Elem = e
    fn right(self) -> Key = k
let t = Two(3, "x")
print(t.left(), t.right())

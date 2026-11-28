// expect: ok, output "YES"
// rule: 05.1
// `self.g ?? (_ => true)` reads `g` and makes a new value: the lambda joins the result's set,
// not `g`'s, so a `Q<Int>` reached through the bound `L: H<S>` needs nothing more in its set.
trait H<S>:
    fn hs(self) -> List<S -> Bool>
struct Q<T>:
    g: ?(T -> Bool)
    fn z2(self, a: T) -> Bool = (self.g ?? (_ => true))(a)
impl<T> H<List<T>> for Q<T>:
    fn hs(self) -> List<List<T> -> Bool> = [s => self.z2(s[0])]
fn run<L: H<S>, S>(q: L, s: S) -> Bool = q.hs()[0](s)
print(yes(run(Q<Int>(None), [3])))

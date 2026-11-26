// expect: ok, output "3\n4 5\n5"
// rule: R08.1.1 R08.1.9
// An associated type is the impl's to say, or the trait's default when the impl leaves it
// out; a bound names it through its parameter, and its own bound holds there.
trait Label:
    fn label(self) -> Int
impl Label for Int:
    fn label(self) -> Int = self + 1
trait Src:
    type Item: Label
    type Count = Int
    fn get(self) -> Self.Item
struct A:
    x: Int
impl Src for A:
    type Item = Int
    fn get(self) -> Int = self.x
fn first<S: Src>(s: S) -> S.Item = s.get()
fn labelled<S: Src>(s: S) -> Int = s.get().label()
fn count<S: Src>(s: S, c: S.Count) -> S.Count = c
print(first(A(3)))
print(labelled(A(3)), labelled(A(4)))
print(count(A(3), 5))

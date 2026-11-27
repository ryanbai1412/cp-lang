// expect: ok, output "5 9"
// rule: R08.2.3
// An impl's bound that leaves its trait's associated type out (`M: Fold2<T>`) is met through
// the impl that fixes it, not the trait's default, so a generic bound reaches that impl's fns.
trait Fold2<T>:
    type Out = ?T
    fn fold2(self, a: T, b: T) -> Self.Out
struct Plus()
impl Fold2<Int> for Plus:
    type Out = Int
    fn fold2(self, a: Int, b: Int) -> Int = a + b
struct Box2<T, M: Fold2<T>>(op: M, a: T)
trait Feed<T>:
    fn feed(self, x: T) -> Int
impl<T, M: Fold2<T>> Feed<T> for Box2<T, M>:
    fn feed(self, x: T) -> Int = 5
fn go<F: Feed<Int>>(f: F) -> Int = f.feed(1)
print(go(Box2(Plus(), 2)), Plus().fold2(4, 5))

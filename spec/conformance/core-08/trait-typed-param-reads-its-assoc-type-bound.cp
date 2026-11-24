// expect: ok, output "5 12"
// rule: 08.2.1 08.1.1
// A trait-typed parameter's associated type has the members its declared bound gives it, at the trait arguments the parameter writes.
trait Ends<V>:
    fn end(self) -> V
trait Walk<V>:
    type Step: Ends<V>
    fn step(self, k: Int) -> Self.Step
struct Hop(to: Int)
impl Ends<Int> for Hop:
    fn end(self) -> Int = self.to
struct Line(k: Int)
impl Walk<Int> for Line:
    type Step = Hop
    fn step(self, k: Int) -> Hop = Hop(self.k + k)
fn far(w: Walk<Int>, k: Int) -> Int = w.step(k).end()
print(far(Line(2), 3), far(Line(4), 8))

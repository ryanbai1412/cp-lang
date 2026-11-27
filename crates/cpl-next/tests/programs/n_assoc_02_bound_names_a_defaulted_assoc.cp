// expect: ok, output "3 4"
// rule: R08.1.9
// A bound that writes all of a trait's parameters still names its defaulted associated type
// through the bounded parameter; each impl picks it, or takes the trait's default.
trait Fo<T>:
    type Fold = ?T
    fn lift(self, x: T) -> Self.Fold
struct Wrap()
impl<T> Fo<T> for Wrap:
    fn lift(self, x: T) -> ?T = Some(x)
struct Plain()
impl<T> Fo<T> for Plain:
    type Fold = T
    fn lift(self, x: T) -> T = x
fn go<M: Fo<T>, T>(x: T, m: M) -> M.Fold = m.lift(x)
print(go(3, Wrap())!, go(4, Plain()))

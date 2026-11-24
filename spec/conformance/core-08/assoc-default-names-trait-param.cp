// expect: ok, output "3 4 5"
// rule: 08.1.9
// A trait's associated-type default may name the trait's own parameter: an impl that leaves `Fold` out gets `?T` for its `T`, one that writes `type Fold = T` overrides it, and an omitted defaulted argument still picks its impl (decision 469).
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
fn go<M: Fo<T>, T>(x: T, m: M = Wrap()) -> M.Fold = m.lift(x)
let a = go(3, Wrap())
let b: Int = go(4, Plain())
let c = go(5)
print(a!, b, c!)

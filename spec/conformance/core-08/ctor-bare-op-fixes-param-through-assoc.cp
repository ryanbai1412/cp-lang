// expect: ok, output "3 3\n3 3"
// rule: 08.1.9
// A constructor's `new<M: Fo<T>, A: Tags<M.Fold, F>>` given a bare `+` takes the `Monoid<T>` impl once `x` binds `T`, so `F` is read off `A`'s impl at `M.Fold = T`; a closure keeps the fn-type impl's `?T` (decision 456).
trait Fo<T>:
    type Fold = ?T
    fn lift(self, x: T) -> Self.Fold
impl<T> Fo<T> for Monoid<T>:
    type Fold = T
    fn lift(self, x: T) -> T = x
impl<T> Fo<T> for (T, T) -> T:
    fn lift(self, x: T) -> ?T = Some(x)
trait Tags<T, F>:
    fn tag(self, x: T) -> F
struct Pairs()
impl<T> Tags<T, (T, T)> for Pairs:
    fn tag(self, x: T) -> (T, T) = (x, x)
struct Box<T, F = Int>(v: T, f: F):
    pub fn new<M: Fo<T>, A: Tags<M.Fold, F>>(op: M, x: T, a: A) -> Box<M.Fold, F>:
        let y = op.lift(x)
        Box<M.Fold, F>(y, a.tag(y))
let b = Box(+, 3, Pairs())
print(b.v, b.f.0)
let c = Box((x: Int, y: Int) => x, 3, Pairs())
print(c.v!, c.f.0!)

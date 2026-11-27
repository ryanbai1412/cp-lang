// expect: ok, output "0 7 0 7"
// rule: R08.1.2
// A fn without `self` of a bound, `T.zero` or `T.f()` on a type parameter, is the impl's for
// the type each instance has.
trait Origin:
    @get fn origin() -> Self
    fn far() -> Self
impl Origin for Int:
    @get fn origin() -> Int = 0
    fn far() -> Int = 7
struct Pt(x: Int)
impl Origin for Pt:
    @get fn origin() -> Pt = Pt(0)
    fn far() -> Pt = Pt(7)
fn ends<T: Origin>(x: T) -> (T, T) = (T.origin, T.far())
let (a, b) = ends(3)
let (c, d) = ends(Pt(1))
print(a, b, c.x, d.x)

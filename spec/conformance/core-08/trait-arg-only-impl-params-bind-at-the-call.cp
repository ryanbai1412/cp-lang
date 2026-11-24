// expect: ok, output "0 YES"
// rule: 08.1.2
// An impl's params that only its trait args name (`T` in `impl<T> Acts<T, T> for Plus`) bind where a bound selects the impl: `A: Acts<T, F>` at `Box(.., act: Plus())` gives `F = T`.
pub trait Acts<T, F>:
    fn tag(self) -> F
struct Plus
struct Asg
impl<T> Acts<T, T> for Plus:
    fn tag(self) -> T = T.zero
impl<T> Acts<T, ?T> for Asg:
    fn tag(self) -> ?T = None
struct Box<T, F = Int>(lz: List<F>, xs: List<T>):
    pub fn new<A: Acts<T, F>>(xs: List<T>, act: A) -> Box<T, F> = Box([act.tag()], [x for x in xs])
let b = Box([1, 2], act: Plus())
let c = Box([1, 2], act: Asg())
print(b.lz[0], yes(c.lz[0] == None))

// expect: E0701
// rule: R08.1.5
// A bound two stated bounds meet, its arguments left open, is ambiguous: never the first bound written.
trait Cv<A>:
    fn cv(self) -> A
impl Cv<Int> for Bool:
    fn cv(self) -> Int = if self: 1 else: 0
fn need<S: Cv<X>, X>(s: S) -> X = s.cv()
fn g<T: Cv<A> + Cv<B>, A, B>(x: T) -> Int:
    need(x) // <- here
    0
print(g<Bool, Int, Int>(true))

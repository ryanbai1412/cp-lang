// expect: E0202
// rule: R08.1.2
// Two blanket impls of one rank overlap even where a type meets both bounds: `Int` has `Tagged` and `Named`, so it would have two choices of `Shown`.
trait Tagged:
    fn tag(self) -> Int
trait Named:
    fn name(self) -> Int
impl Tagged for Int:
    fn tag(self) -> Int = 1
impl Named for Int:
    fn name(self) -> Int = 2
trait Shown:
    fn shown(self) -> Int
impl<T: Tagged> Shown for T:
    fn shown(self) -> Int = 1
impl<T: Named> Shown for T: // <- here
    fn shown(self) -> Int = 2
print(5.shown())

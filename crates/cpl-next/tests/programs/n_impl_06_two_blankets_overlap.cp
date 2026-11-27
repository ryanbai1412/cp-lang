// expect: E0202
// rule: R08.1.2
// Two blanket impls of one trait for bare parameters overlap, whatever their bounds.
trait Tagged:
    fn tag(self) -> Int
trait Named:
    fn name(self) -> Int
trait Shown:
    fn shown(self) -> Int
impl<T: Tagged> Shown for T:
    fn shown(self) -> Int = 1
impl<T: Named> Shown for T: // <- here
    fn shown(self) -> Int = 2

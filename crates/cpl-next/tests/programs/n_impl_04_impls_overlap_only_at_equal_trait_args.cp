// expect: E0202
// rule: R08.1.2
// Impls of one trait for one type overlap when their trait arguments can be one type too.
trait Tag<T>:
    fn tag(self) -> Int
impl Tag<Int> for Bool:
    fn tag(self) -> Int = 1
impl Tag<Str> for Bool:
    fn tag(self) -> Int = 2
impl<T> Tag<T> for Bool: // <- here
    fn tag(self) -> Int = 0

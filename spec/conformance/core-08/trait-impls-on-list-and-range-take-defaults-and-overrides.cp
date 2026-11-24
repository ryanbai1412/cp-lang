// expect: ok, output "6 1000 4 1000"
// rule: 08.1.4
// Impls for List<Int> and Range<Int> are found by the type's head, in generic code and on a
// value: the List impl takes the trait's default `tot`, the Range impl overrides it.
trait Tot:
    fn tot(self) -> Int = 1000
    fn total(self) -> Int
impl Tot for List<Int>:
    fn total(self) -> Int = self[0] + self[1] + self[2]
impl Tot for Range<Int>:
    fn tot(self) -> Int = self.len
    fn total(self) -> Int = 0
fn via<S: Tot>(s: S) -> (Int, Int) = (s.total(), s.tot())
print(via([1, 2, 3]).0, [1, 2, 3].tot(), (0..4).tot(), via([1, 2, 3]).1)

// expect: E0201
// rule: 08.1.10
// An impl on `type_of` of a name nothing defines points at that name, with the nearest names.
fn mx(a: Int, b: Int) -> Int = if a > b: a else: b
trait Named:
    fn name(self) -> Str
impl Named for type_of(mxx): // <- here
    fn name(self) -> Str = "mx"
print(mx(1, 2))

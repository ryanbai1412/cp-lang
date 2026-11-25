// expect: E0202
// rule: 08.1.11
// R08.1.11: two impls of one trait for one fn item are E0202.
trait Op:
    fn go(self) -> Int
fn mx(a: Int, b: Int) -> Int = if a > b: a else: b
impl Op for type_of(mx):
    fn go(self) -> Int = 1
impl Op for type_of(mx): // <- here
    fn go(self) -> Int = 2
print(1)

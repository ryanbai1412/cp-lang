// expect: ok, output "YES"
// rule: 08.2.5
// A type alias is a receiver like the type it names: its trait's static fns,
// and a struct's own fns before a trait's.
trait Z:
    fn z() -> Self
impl Z for _..=_:
    fn z() -> Self = Self.nth(1)
struct P:
    a: Int
    fn new(a: Int) -> P = P(a: a)
type Q = P
type Small = 0..=3
print(yes(Small.z() == 1 and Q.new(7).a == 7))

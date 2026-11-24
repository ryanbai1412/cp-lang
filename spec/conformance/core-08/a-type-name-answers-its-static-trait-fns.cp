// expect: ok, output "YES"
// rule: 08.2.5
// A trait's static fn is called on the type's name, any type alike: the
// receiver resolves through the type table, built-in scalar or user struct.
trait Z:
    fn z() -> Self
impl Z for Bool:
    fn z() -> Bool = true
impl Z for Int:
    fn z() -> Int = 3
impl Z for Char:
    fn z() -> Char = 'a'
struct P(a: Int)
impl Z for P:
    fn z() -> P = P(7)
print(yes(Bool.z() and Int.z() == 3 and Char.z() == 'a' and P.z().a == 7))

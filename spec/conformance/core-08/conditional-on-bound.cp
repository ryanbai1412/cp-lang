// expect: ok, output "fancy plain"
// rule: 08.2.9
trait Fancy
struct A: x: Int
struct B: x: Int
impl Fancy for A
fn kind<T>(x: T) -> Str:
    if T: Fancy:
        return "fancy"
    return "plain"
print(kind(A(1)), kind(B(1)))

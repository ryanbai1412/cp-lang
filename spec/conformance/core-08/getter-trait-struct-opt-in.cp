// expect: ok, output "7"
// rule: 08.1.7
// A bodiless `@get` member is a field requirement: a bodiless `impl` is met by the struct's field.
trait Named:
    @get fn name(self) -> Int
fn g<A: Named>(a: A) -> Int = a.name
struct T: name: Int
impl Named for T
print(g(T(7)))

// expect: ok, output "3 7\n12 ab"
// rule: R12.11.3
// A trait of bodiless `@get` members is met by any record with those fields, of those types, and
// by a struct through a bodiless impl.
trait Wt:
    @get fn w(self) -> Int
trait Target<V>:
    @get fn to(self) -> V
fn wt<T: Wt>(x: T) -> Int = x.w
fn dest<E: Target<V>, V>(e: E) -> V = e.to
struct Road:
    w: Int
    len: Int
impl Wt for Road
print(wt((w: 3, to: 1)), dest((to: 7, w: 2)))
print(wt(Road(12, 5)), dest((to: "ab")))

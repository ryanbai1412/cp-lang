// expect: ok, output "1 2\n2 1\n3"
// A trait's self-less fn returning `Self`, named where no fn is, takes its impl from a written
// type argument or the expected type: `make<A>()` is `A.make()` (R08.2.8).
trait Make:
    fn make() -> Self
struct A: n: Int
struct B: n: Int
impl Make for A:
    fn make() -> A = A(1)
impl Make for B:
    fn make() -> B = B(2)
fn twice<T: Make>() -> (T, T) = (make<T>(), make())
let a = make<A>()
let b: B = make()
print(a.n, b.n)
let (x, y) = twice<B>()
print(x.n, a.n)
print(make<A>().n + make<B>().n)

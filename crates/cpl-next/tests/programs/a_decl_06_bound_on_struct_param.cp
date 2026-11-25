// expect: ok, output "2"
// rule: R08
// A trait bound written on a struct's type parameter is checked: Int has no impl => E0606; here a struct with the impl.
trait Sz:
    fn sz(self) -> Int
struct A:
    n: Int
impl Sz for A:
    fn sz(self) -> Int = n
struct Wrap<T: Sz>:
    v: T
    fn size(self) -> Int = v.sz()
print(Wrap(A(2)).size())

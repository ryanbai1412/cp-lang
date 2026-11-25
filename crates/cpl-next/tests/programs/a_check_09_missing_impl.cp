// expect: E0606
// rule: diagnostics E0606
trait Sz:
    fn sz(self) -> Int
struct A:
    n: Int
impl Sz for A:
    fn sz(self) -> Int = n
fn size<T: Sz>(v: T) -> Int = v.sz()
print(size(5))  // <- here

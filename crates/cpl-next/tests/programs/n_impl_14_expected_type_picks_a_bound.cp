// expect: ok, output "1"
// rule: R08.1.4
// One trait a type parameter is bounded by at two arguments: the expected type picks the bound.
trait Cv<U>:
    fn cv(self) -> U
impl Cv<Int> for Int:
    fn cv(self) -> Int = self
impl Cv<Bool> for Int:
    fn cv(self) -> Bool = self > 0
fn f<T: Cv<Int> + Cv<Bool>>(x: T) -> Bool = x.cv()
print(if f(5): 1 else: 0)

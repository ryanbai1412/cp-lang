// expect: ok, output "1 s"
// rule: R08.1.2
// Of two impls of one trait for a type, the expected type of the call picks the one that fits.
trait Cv<U>:
    fn cv(self) -> U
impl Cv<Bool> for Int:
    fn cv(self) -> Bool = self > 0
impl Cv<Str> for Int:
    fn cv(self) -> Str = "s"
let a: Bool = 5.cv()
let b: Str = 5.cv()
print(if a: 1 else: 0, b)

// expect: E0701
// rule: R08.1.2
// Two impls that fit a call nothing narrows are no choice, not the first one declared.
trait Cv<U>:
    fn cv(self) -> U
impl Cv<Bool> for Int:
    fn cv(self) -> Bool = self > 0
impl Cv<Str> for Int:
    fn cv(self) -> Str = "s"
print(5.cv())  // <- here

// expect: E0701
// rule: R08.1.4
// A method two traits give a type is no choice: the call names its trait.
trait P:
    fn hi(self) -> Int
trait Q:
    fn hi(self) -> Int
impl P for Int:
    fn hi(self) -> Int = 1
impl Q for Int:
    fn hi(self) -> Int = 2
print(5.hi())  // <- here

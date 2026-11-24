// expect: E0701
// rule: 08.1.4
trait A:
    fn m(self) -> Int
trait B:
    fn m(self) -> Int
struct S: x: Int
impl A for S:
    fn m(self) -> Int = 1
impl B for S:
    fn m(self) -> Int = 2
fn use_it(s: S) -> Int = s.m() // <- here
print(use_it(S(0)))

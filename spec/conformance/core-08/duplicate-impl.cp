// expect: E0202 at 6
// rule: 08.1.2
trait A:
    fn a(self) -> Int
struct S: x: Int
impl A for S:
    fn a(self) -> Int = 1
impl A for S:
    fn a(self) -> Int = 2

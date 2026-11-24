// expect: E0202
// rule: 08.1.2
trait A:
    fn a(self) -> Int
struct S: x: Int
impl A for S:
    fn a(self) -> Int = 1
impl A for S: // <- here
    fn a(self) -> Int = 2

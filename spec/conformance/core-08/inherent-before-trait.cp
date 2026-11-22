// expect: ok, output "1"
// rule: 08.1.4
trait A:
    fn m(self) -> Int
struct S:
    x: Int
    fn m(self) -> Int = 1
impl A for S:
    fn m(self) -> Int = 2
print(S(0).m())

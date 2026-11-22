// expect: ok, output "1 2"
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
print(A.m(S(0)), B.m(S(0)))

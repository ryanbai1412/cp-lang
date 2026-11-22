// expect: ok, output "3"
// rule: 08.1.2
trait A:
    fn a(self) -> Int = 1
trait B:
    fn b(self) -> Int = 2
struct S: x: Int
impl A + B for S
let s = S(0)
print(s.a() + s.b())

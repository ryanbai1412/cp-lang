// expect: ok, output "A3 [A3]"
// rule: 10.3.5
struct A(x: Int)
impl Show for A:
    fn show(self) -> Str = "A{self.x}"
let a = A(3)
print(a, "[{a}]")

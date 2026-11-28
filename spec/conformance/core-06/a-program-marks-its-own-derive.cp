// expect: ok, output "2 4"
// engines: next
// rule: 06.4.12
// R06.4.12: any module may write a marker decorator core uses; a program's `@derive` trait is
// derived field by field for a struct whose fields all have it, as core's are.
@derive
trait Twin:
    fn twin(self) -> Self
impl Twin for Int:
    fn twin(self) -> Int = self * 2
struct P:
    a: Int
    b: Int
let p = P(1, 2).twin()
print(p.a, p.b)

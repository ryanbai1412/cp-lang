// expect: ok, output "5.0000000000 6 1.5000000000 4"
// rule: 08.2
// A trait implemented for `F64` satisfies a generic bound just as one for `Int` does, associated types included.
trait Twice:
    type Found
    fn twice(self) -> Self
    fn found(self) -> Self.Found
impl Twice for F64:
    type Found = ?F64
    fn twice(self) -> F64 = self * 2.0
    fn found(self) -> ?F64 = Some(self)
impl Twice for Int:
    type Found = Int
    fn twice(self) -> Int = self * 2
    fn found(self) -> Int = self
fn go<D: Twice>(x: D) -> D = x.twice()
fn get<D: Twice>(x: D) -> D.Found = x.found()
print(go(2.5), go(3), get(1.5)!, get(4))

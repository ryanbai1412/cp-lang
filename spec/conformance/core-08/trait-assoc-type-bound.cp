// expect: ok, output "3"
// rule: 08.1.1
trait Keyed:
    type K: Ord
    fn key(self) -> Self.K
struct P(a: Int)
impl Keyed for P:
    type K = Int
    fn key(self) -> Int = self.a
print(P(3).key())

// expect: E0606 at 7
// rule: 08.1.1
// An impl's choice for an associated type must have the traits the trait's bound names.
trait Keyed:
    type K: Ord
    fn key(self) -> Self.K
struct P(a: Int)
impl Keyed for P:
    type K = Int -> Int
    fn key(self) -> Int -> Int = x => x + self.a
print(P(3).key()(1))

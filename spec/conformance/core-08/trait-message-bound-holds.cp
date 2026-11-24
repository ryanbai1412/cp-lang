// expect: ok, output "7"
// rule: 08.1.8
// A trait's attached message changes nothing when the bound holds.
@on_unimplemented("Scorer needs fn score(self) -> Int")
trait Scorer:
    fn score(self) -> Int
struct P:
    v: Int
impl Scorer for P:
    fn score(self) -> Int = self.v
fn total<T: Scorer>(x: T) -> Int = x.score()
print(total(P(v: 7)))

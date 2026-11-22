// expect: E0606 at 5
// rule: 08.1.8
#on_unimplemented("Scorer needs fn score(self) -> Int")
trait Scorer:
    fn score(self) -> Int
fn total<T: Scorer>(x: T) -> Int = x.score()
print(total(5))

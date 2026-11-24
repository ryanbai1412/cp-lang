// expect: ok, output "3 3\n1 1\n4 4\n3 3"
// rule: 08.2.2
// `map`'s own `U` is fresh at each call: `W<A, U>`'s `U` (here `Int`) never binds it, so the lambda gives back a `List<Int>`.
use std.seq.{Seq}
pub struct W<A, U>(pub src: List<U>)
impl Seq<U> for W<A, U>:
    fn iter(self) -> yields U:
        for x in self.src: yield x
fn ap<S: Seq<T>, T, U>(s: S, g: T -> U) -> List<U> = [g(x) for x in s]
let w: W<Int, Int> = W([3, 1, 4])
print(List(w.map(x => [x, x])))
print(ap(w, x => [x, x])[0])

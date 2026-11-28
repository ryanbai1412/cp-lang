// expect: E0606
// rule: R04.8.11
// A member only `Array` gives is not one a `List` has, so `[D]T` has it only where `D: Domain`.
trait Tag:
    fn tag(self) -> Int
impl<D: Domain, T> Tag for Array<D, T>:
    fn tag(self) -> Int = 1
fn f<D: Discrete, T>(a: [D]T) -> Int = a.tag()
print(f<Int, Int>([4]))

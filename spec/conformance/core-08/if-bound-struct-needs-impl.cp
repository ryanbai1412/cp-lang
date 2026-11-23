// expect: ok, output "0 7"
// rule: 08.1.7 08.2.9
// A named struct is nominal: matching fields alone do not meet a trait, in `if A: Trait` as in a bound.
trait Named:
    @get fn name(self) -> Int
struct S: name: Int
struct T: n: Int
impl Named for T:
    @get fn name(self) -> Int = self.n
fn f<A>(a: A) -> Int:
    if A: Named:
        return a.name
    0
print(f(S(6)), f(T(7)))

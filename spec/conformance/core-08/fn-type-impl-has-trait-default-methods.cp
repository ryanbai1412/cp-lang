// expect: ok, output "5 20 YES"
// rule: 08.1.7
// An impl for a fn type has its trait's default methods and getters as its own, as any other impl does.
trait Twice<T>:
    fn ap(self, x: T) -> T
    fn twice(self, x: T) -> T = self.ap(self.ap(x))
    @get fn grows(self) -> Bool = self.ap(T(0)) > T(0)
impl<T> Twice<T> for T -> T:
    fn ap(self, x: T) -> T = self(x)
let inc = (x: Int) => x + 1
print(inc.twice(3), ((x: Int) => x * 2).twice(5), yes(inc.grows))

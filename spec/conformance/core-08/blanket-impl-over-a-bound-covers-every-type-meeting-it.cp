// expect: ok, output "8 3 7 5 30"
// rule: 08.1.9
// One blanket impl gives `Pick` to every `Indexed`, a list and a table alike, beside an
// impl for fn types: each value takes the one impl whose bounds it meets.
trait Pick<T, U>:
    fn pick(self, x: T) -> U
    fn twice(self, x: T) -> (U, U) = (self.pick(x), self.pick(x))
impl<S: Indexed<D, U>, D, U> Pick<D, U> for S:
    fn pick(self, x: D) -> U = self[x]
impl<T, U> Pick<T, U> for T -> U:
    fn pick(self, x: T) -> U = self(x)
fn via<B: Pick<T, U>, T, U>(b: B, x: T) -> U = b.pick(x)
enum E:
    A
    B
let t: [E]Int = [1, 3]
let f = (x: Int) => x + 1
print(via([7, 8], 1), via(t, E.B), [7, 8].pick(0), via(f, 4), t.twice(E.B).0 * 10)

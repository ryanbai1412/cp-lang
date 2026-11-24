// expect: E0606
// rule: 08.1.9
// `Int` is no `Indexed`, so the blanket impl gives it no `Pick`.
trait Pick<T, U>:
    fn pick(self, x: T) -> U
impl<S: Indexed<D, U>, D, U> Pick<D, U> for S:
    fn pick(self, x: D) -> U = self[x]
fn via<B: Pick<T, U>, T, U>(b: B, x: T) -> U = b.pick(x)
print(via(5, 1)) // <- here

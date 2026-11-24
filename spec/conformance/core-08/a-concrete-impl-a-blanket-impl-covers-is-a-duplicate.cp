// expect: E0202
// rule: 08.1.9
// `List<Int>` is an `Indexed`, so the blanket impl already gives it `Pick`: two impls of
// one trait apply to it.
trait Pick<T, U>:
    fn pick(self, x: T) -> U
impl<S: Indexed<D, U>, D, U> Pick<D, U> for S:
    fn pick(self, x: D) -> U = self[x]
impl Pick<Int, Int> for List<Int>: // <- here
    fn pick(self, x: Int) -> Int = 0
print([1].pick(0))

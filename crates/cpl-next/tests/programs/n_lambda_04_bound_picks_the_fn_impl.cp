// expect: ok, output "9\n7\n2"
// A lambda given for a trait-bound parameter is the fn type the trait's impl targets
// (`impl<T> Pick<T> for (T, T) -> T`), so its parameters need no annotation; a fn value of that
// type meets the bound through the same impl.
trait Pick<T>:
    fn pick(self, a: T, b: T) -> T
impl<T> Pick<T> for (T, T) -> T:
    fn pick(self, a: T, b: T) -> T = self(a, b)
fn best<T, P: Pick<T>>(xs: List<T>, p: P) -> T:
    var b = xs[0]
    for x in xs: b = p.pick(b, x)
    b
print(best([3, 9, 2], (a, b) => if a > b: a else: b))
let add: (Int, Int) -> Int = (a, b) => a + b
print(best([1, 2, 3], add))
print(best([5, 2, 7], (a, b) => if a < b: a else: b))

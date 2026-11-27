// expect: ok, output "81 4 twice 9 5"
// rule: R08.1.10 R08.1.11
// `type_of(f)` is the fn item's own type: a parameter of it calls `f`, and an impl for it is
// the one a bound picks when `f` itself is passed.
fn sq(x: Int) -> Int = x * x
fn twice(f: type_of(sq), x: Int) -> Int = f(f(x))
fn half(x: Int) -> Int = x / 2
fn on(f: type_of(half), x: Int) -> Int = f(x)
trait Named:
    fn label(self) -> Str
fn dbl(a: Int, b: Int) -> Int = 2 * a + b
impl Named for type_of(dbl):
    fn label(self) -> Str = "twice"
fn tag<M: Named>(m: M) -> Str = m.label()
fn apply(f: (Int, Int) -> Int, x: Int) -> Int = f(x, x)
fn mx(a: Int, b: Int) -> Int = if a > b: a else: b
impl Op<Int> for type_of(mx):
    fn op(self, a: Int, b: Int) -> Int = mx(a, b)
fn both<M: Op<Int>>(m: M, a: Int, b: Int) -> Int = m.op(a, b)
print(twice(sq, 3), on(half, 9), tag(dbl), apply(dbl, 3), both(mx, 2, 5))

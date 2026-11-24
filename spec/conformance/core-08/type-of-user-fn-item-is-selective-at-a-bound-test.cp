// expect: ok, output "1 0 0 5"
// rule: 08.1.11 08.2.9
// `mx`'s own impl makes it `Selective`; `mn` and a closure of the same fn type are not.
fn mx(a: Int, b: Int) -> Int = if a > b: a else: b
fn mn(a: Int, b: Int) -> Int = if a < b: a else: b
impl Op<Int> for type_of(mx):
    fn op(self, a: Int, b: Int) -> Int = mx(a, b)
impl Idempotent<Int> for type_of(mx)
impl Selective<Int> for type_of(mx)
fn pick<T, M: Op<T>>(m: M, a: T, b: T) -> Int = if M: Selective<T>: 1 else: 0
fn apply(f: (Int, Int) -> Int, a: Int, b: Int) -> Int = f(a, b)
print(pick(mx, 1, 2), pick(mn, 1, 2), pick((a: Int, b: Int) => a + b, 1, 2), apply(mx, 2, 5))

// expect: ok, output "assoc YES YES NO 3 3 3"
// rule: 08.3.5
// A trait's laws are a derived `laws` default: each impl's copy binds `T` to the impl's arg and runs
// the impl's fns, so `-` fails the associativity that `+` and `max` keep.
trait Op<T>:
    fn op(a: T, b: T) -> T
    law assoc(a: T, b: T, c: T) where T: Eq = op(op(a, b), c) == op(a, op(b, c))
struct Plus
impl Op<Int> for Plus:
    fn op(a: Int, b: Int) -> Int = a + b
struct Mx
impl Op<Int> for Mx:
    fn op(a: Int, b: Int) -> Int = if a > b: a else: b
struct Minus
impl Op<Int> for Minus:
    fn op(a: Int, b: Int) -> Int = a - b
let s = [1, 2, 5]
let p = [0, 1, 2]
print(
    Plus().laws[0].name,
    yes(Plus().laws[0].holds(s, p)),
    yes(Mx().laws[0].holds(s, p)),
    yes(Minus().laws[0].holds(s, p)),
    Minus().laws[0].sizes(s),
)

// expect: ok, output "1\n 1 1 5\n998244352 1"
// rule: 02.1.2
// A local two items of one tuple read, dead after it, is still read by both:
// neither item may take it away from the other.
// copies: 2
use std.poly.{gcd_poly}
fn f(q: List<Int>) -> (List<Int>, List<Int>, List<Int>, List<Int>):
    let one: List<Int> = [1]
    let zero: List<Int> = []
    let step = (zero, one, one, q)
    print(step.1.len)
    step
print(f([5]))
type Z = Mod<998_244_353>
let a: List<Z> = [2, -3, 1]
let b: List<Z> = [3, -4, 1]
print(gcd_poly(a, b))

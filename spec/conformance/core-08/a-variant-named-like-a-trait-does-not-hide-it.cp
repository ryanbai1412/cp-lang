// expect: ok, output "4 3 rand"
// rule: 08.1.1
// A name where only a trait can stand (an impl's trait, a bound) resolves among traits, so a variant `Random` beside std's trait `Random` hides neither.
enum Shape = Line | Random
struct W:
    x: Int
impl Random for W:
    fn gen() -> W = W(4)
fn drawn<T: Random>() -> T = gen<T>()
let s = Random
print(drawn<W>().x, W(3).x, if s == Random: "rand" else: "line")

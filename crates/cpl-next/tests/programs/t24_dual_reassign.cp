// expect: E0000
// (pending) a diagnostic code for: built from its own old value
// input: "1\n"
// The same with one variable reassigned. Runs today through std::function.
struct Monoid:
    id: Int
    op: (Int, Int) -> Int
fn dual(m: Monoid) -> Monoid = Monoid(m.id, (a, b) => m.op(b, a))
let n: Int = read()
var m = Monoid(0, (a, b) => a * 10 + b)
if n % 2 == 1: m = dual(m)
print(m.op(1, 2))

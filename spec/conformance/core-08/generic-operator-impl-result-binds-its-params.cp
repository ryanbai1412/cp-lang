// expect: ok, output "5 6 8"
// rule: 08.1.5
// A generic struct's operator impl gives its result at the operands' own type
// arguments: `impl Add for W<T>` on two `W<Int>` is a `W<Int>`, so members read
// off the parenthesised result are typed.
struct W<T>(v: T)
impl Add for W<T>:
    fn add(a: W<T>, b: W<T>) -> W<T> = a
fn total(w: W<Int>, k: Int) -> Int = w.v + k
let w = W(2)
print((w + W(9)).v + 3, total(w + w, 4), (W(4) + W(1)).v * 2)

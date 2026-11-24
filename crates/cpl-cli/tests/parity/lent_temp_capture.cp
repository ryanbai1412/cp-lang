// A closure that keeps a view, or makes one over temporaries, reads live values.
use std.seq.{zip}
struct H: xs: List<Int>
fn wrap(xs: List<Int>) -> lends H = H(xs)
let n: Int = read()
let a = [i for i in 0..n]
let z = zip(a, [i * 10 for i in 0..n])
let at = (k: Int) => z.nth(k).1
let h = wrap([i + 100 for i in 0..n])
let first = () => h.xs[0]
let made = (k: Int) => zip(a, [k for _ in 0..k]).len + wrap([k, k]).xs.len
print(at(n - 1), first(), made(1), made(n + 3))

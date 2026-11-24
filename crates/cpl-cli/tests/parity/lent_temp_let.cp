// A `let` keeps a view made over temporaries: each temporary lives as long as the view.
use std.seq.{zip}
struct H: xs: List<Int>
fn wrap(xs: List<Int>) -> lends H = H(xs)
fn go(n: Int) -> Int:
    let z = zip([i * 2 for i in 0..n], [i for i in 0..n + 1])
    let m = zip([i for i in 0..n], [7 for _ in 0..n]).map((x, y) => x * y)
    let h = wrap([i * i for i in 0..n])
    z.len + z.nth(n - 1).0 + m.sum() + h.xs[n - 1]
let n: Int = read()
let w = zip([i for i in 0..n], [9, 8])
print(go(n), go(n + 1), w.len, w.nth(1).1)

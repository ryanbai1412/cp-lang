// A view returned past its statement owns what a temporary would have lent it.
use std.seq.{zip}
struct H: xs: List<Int>
fn wrap(xs: List<Int>) -> lends H = H(xs)
fn mk(n: Int) -> H = wrap([i * 5 for i in 0..n])
fn pick(n: Int) -> H:
    if n % 2 == 0: return wrap([i for i in 0..n])
    wrap([n for _ in 0..n])
let n: Int = read()
let a = mk(n)
let b = pick(n)
let c = pick(n + 1)
print(a.xs[n - 1], b.xs.len, c.xs[0], zip(mk(n).xs, [1, 2]).len)

// expect: ok, output "1 0 1 0 1\n0 1"
// rule: R02.4.5
// A `?T` compared with the `None` value tests presence whatever the expression's shape: `None`,
// `(None)` or a `let` bound to it, on either side. `T` needs no `Eq` (R08.1.5).
fn inc(x: Int) -> Int = x + 1
fn b(x: Bool) -> Int = if x: 1 else: 0
fn mk(k: Int) -> ?(Int -> Int):
    if k > 0:
        return inc
    None
let c = mk(0)
let d = mk(1)
let n: ?(Int -> Int) = None
print(b(c == None), b(d == None), b(c == n), b(n == d), b((None) != d))
print(b(None == d), b(n != d))

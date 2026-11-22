// error: expected Int, got ?Int
// Anti-spookiness Invariant 3: passing `mut x` to a function invalidates the narrowing on `x`.
fn reset(mut v: ?Int):
    v = none

var x: ?Int = 5
if x != none:
    reset(mut x)
    let y: Int = x
    print(y)

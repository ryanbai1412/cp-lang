// error: `y` is declared an Int, but `x` is an ?Int
// Anti-spookiness Invariant 3: passing `mut x` to a function invalidates the narrowing on `x`.
fn reset(mut v: ?Int):
    v = None

var x: ?Int = 5
if x != None:
    reset(mut x)
    let y: Int = x
    print(y)

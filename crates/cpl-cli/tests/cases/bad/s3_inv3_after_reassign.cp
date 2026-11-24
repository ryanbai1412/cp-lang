// error: `y` is declared an Int, but `x` is an ?Int
// Anti-spookiness Invariant 3: mutating a narrowed local invalidates the narrowing immediately.
fn maybe() -> ?Int = None
var x: ?Int = 5
if x != None:
    x = maybe()
    let y: Int = x
    print(y)

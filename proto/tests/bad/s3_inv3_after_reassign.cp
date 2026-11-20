// error: expected Int, got ?Int
// Anti-spookiness Invariant 3: mutating a narrowed local invalidates the narrowing immediately.
fn maybe() -> ?Int = none
var x: ?Int = 5
if x != none:
    x = maybe()
    let y: Int = x
    print(y)

// error: expected Int, got ?Int
// Anti-spookiness Invariant 3: calling a function that writes a captured local invalidates its narrowing.
var x: ?Int = 5
fn clear_x():
    x = None

if x != None:
    clear_x()
    let y: Int = x
    print(y)

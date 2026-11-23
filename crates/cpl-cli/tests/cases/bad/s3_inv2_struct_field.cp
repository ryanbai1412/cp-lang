// error: expected Int, got ?Int
// Anti-spookiness Invariant 2: struct fields (`b.x`) are NEVER narrowed, only local bindings.
struct Box:
    x: ?Int

let b = Box(x: 42)
if b.x != None:
    let y: Int = b.x
    print(y)

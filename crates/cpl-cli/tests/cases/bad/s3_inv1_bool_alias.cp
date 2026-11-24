// error: `y` is declared an Int, but `x` is an ?Int
// Anti-spookiness Invariant 1: narrowing comes ONLY from the syntactic guard, never an aliased Bool.
let x: ?Int = 5
let ok = x != None
if ok:
    let y: Int = x
    print(y)

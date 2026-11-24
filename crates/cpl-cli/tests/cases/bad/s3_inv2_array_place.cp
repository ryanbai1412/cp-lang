// error: `y` is declared an Int, but `a[i]` is an ?Int
// Anti-spookiness Invariant 2: array elements (`a[i]`) are NEVER narrowed, only local bindings.
let n: 1..=10 = read()
domain I = 1..=n
let a: [I]?Int = None
let i = I.first!
if a[i] != None:
    let y: Int = a[i]
    print(y)

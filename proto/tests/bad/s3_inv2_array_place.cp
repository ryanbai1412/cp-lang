// error: expected Int, got ?Int
// Anti-spookiness Invariant 2: array elements (`a[i]`) are NEVER narrowed, only local bindings.
let n: 1..=10 = read()
domain I = 1..=n
let a: [I]?Int = none
let i = I.first
if a[i] != none:
    let y: Int = a[i]
    print(y)

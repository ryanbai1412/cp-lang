// error: `!` takes the value out of an optional, but this is a `Int`
// Anti-spookiness Invariant 4: `!` on a genuinely non-optional type is still a hard compile error.
let x: Int = 5
print(x!)

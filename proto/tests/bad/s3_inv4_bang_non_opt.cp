// error: `!` on a non-optional
// Anti-spookiness Invariant 4: `!` on a genuinely non-optional type is still a hard compile error.
let x: Int = 5
print(x!)

// expect: E0103 at 3
// rule: 01.10.1
// A generator is only a call argument, never a parenthesized value; the fix is `max(x for x in xs)`.
let xs = [3, 1, 2]
print((x for x in xs).max()!)

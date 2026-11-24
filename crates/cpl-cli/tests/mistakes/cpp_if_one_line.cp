// here: 4
// C's `if (c) stmt` (self-run mistakes): the fix put the `:` before the body, as `if (c) :stmt`.
let n = 4
if (n % 2 == 0) print(n)

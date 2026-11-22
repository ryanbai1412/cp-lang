// expect: R0101 at 4
// rule: 04a.11.4
use std.strings.{z_function}
let s = "abab"
let z = z_function(s[2..])
print(z[0])

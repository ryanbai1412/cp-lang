// expect: ok, output "2\n4"
// rule: 04a.6.2 04a.11.4 04b.1.2 04b.5.3
use std.strings.{z_function}
let s = "abaab"
let z = z_function(s[1..])
var c = 0
for i in s[1..].keys:
    if z[i] >= 1: c += 1
print(c)
print(z[1])

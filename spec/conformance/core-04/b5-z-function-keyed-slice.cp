// expect: ok, output "204 300 402 500"
// rule: 04b.5.3
use std.strings.{z_function}
let s = "xxabab"
print(z_function(s[2..]).items.map((k, x) => k * 100 + x).collect())

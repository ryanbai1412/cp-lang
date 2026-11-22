// expect: ok, output "3"
// rule: 04b.5.5
use std.strings.{lyndon_factorization}
print(lyndon_factorization("xba"[1..]).sum())

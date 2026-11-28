// expect: ok, output "6 ab"
// engines: next
use std.laws.{Sample}
print(Str.samples().len, Str.samples()[3])

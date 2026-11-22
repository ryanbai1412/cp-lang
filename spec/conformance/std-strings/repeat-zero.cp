// expect: ok, output "0 3"
use std.strings.{repeat}
print("ab".repeat(0).len, "x".repeat(3).len)

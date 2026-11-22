// expect: ok, output "YES YES"
use std.strings.{manacher}
let p = manacher("abc")
print(yes(p.is_palindrome(1..1)), yes(p.is_palindrome(0..1)))

// expect: ok, output "YES NO"
use std.strings.{manacher}
let p = manacher("ab")
print(yes(p.is_palindrome(1..1)), yes(p.is_palindrome(0..2)))

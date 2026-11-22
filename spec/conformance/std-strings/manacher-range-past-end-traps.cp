// expect: R0401 at 3 (checked)
use std.strings.{manacher}
let p = manacher("abc")
print(yes(p.is_palindrome(2..5)))

// expect: ok, output "YES YES NO"
use std.strings.{is_palindrome}
print(yes(is_palindrome("")), yes(is_palindrome("a")), yes(is_palindrome("ab")))

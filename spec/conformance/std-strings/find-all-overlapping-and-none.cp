// expect: ok, output "0 2\n0"
use std.strings.{find_all}
print("ababa".find_all("aba").collect())
print("abc".find_all("abcd").count())

// expect: ok, output "YES YES YES YES\nYES YES YES YES"
use std.strings.{manacher}
print(manacher("aba").len == 3, manacher("").is_empty, manacher("abacaba").longest_at(3) == 7, manacher("abba").longest_between(2) == 4)
print(manacher("aaa").count() == 6, manacher("abba").count() == 6, manacher("abacdc").longest() == (l: 0, r: 3), manacher("").longest() == (l: 0, r: 0))

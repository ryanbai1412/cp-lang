// expect: ok, output "YES YES YES YES\nYES YES YES YES"
use std.strings.{manacher}
print(yes(manacher("aba").len == 3), yes(manacher("").is_empty), yes(manacher("abacaba").longest_at(3) == 7), yes(manacher("abba").longest_between(2) == 4))
print(yes(manacher("aaa").count() == 6), yes(manacher("abba").count() == 6), yes(manacher("abacdc").longest() == (l: 0, r: 3)), yes(manacher("").longest() == (l: 0, r: 0)))

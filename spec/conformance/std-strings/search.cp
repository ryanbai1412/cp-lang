// expect: ok, output "2 2 none\n3 5 2\nYES NO YES YES NO\n0\n1\n2\n2\n3\n1"
use std.strings.{borders}
print("banana".search("nan"), "banana".search('n'), "banana".search("x"))
print("banana".rsearch("an"), "banana".rsearch('a'), "ab".rsearch(""))
print(yes([1, 2, 3].ends_with([2, 3])), yes("banana".ends_with("nab")), yes("banana".starts_with("ban")), yes("banana".starts_with("nan", at: 2)), yes([1, 2, 3].starts_with([2])))
for i in "aaaa".find_all("aa"): print(i)
print(period("abab"))
for b in borders("abacaba"): print(b)

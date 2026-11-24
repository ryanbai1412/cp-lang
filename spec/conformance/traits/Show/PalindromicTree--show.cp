// expect: ok, output "a a b: 3 palindromes"
use std.strings.{PalindromicTree}
var t = PalindromicTree<Char>()
t.push('a')
t.push('a')
t.push('b')
print(t)

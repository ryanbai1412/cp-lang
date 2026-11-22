// expect: ok, output "2 0"
use std.strings.{Trie}
var t = Trie<Char>()
t.add("ab")
t.add("ac")
print(t.count_prefix(""), t.count_prefix("b"))

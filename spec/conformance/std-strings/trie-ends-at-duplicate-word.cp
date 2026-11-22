// expect: ok, output "2 2"
use std.strings.{Trie}
var t = Trie<Char>()
let v = t.add("ab")
t.add("ab")
print(t.ends_at(v), t.pass_at(1))

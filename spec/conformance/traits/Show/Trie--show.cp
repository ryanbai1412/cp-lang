// expect: ok, output "a ab b b"
use std.strings.{Trie}
var t = Trie<Char>()
for w in ["b", "ab", "a", "b"]: t.add(w)
print(t)

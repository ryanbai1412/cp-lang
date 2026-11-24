// expect: ok, output "1 5 5"
use std.bitset.{BitTrie}
var t = BitTrie<3>.of(5)
t.add(1)
t.add(5)
print(t)

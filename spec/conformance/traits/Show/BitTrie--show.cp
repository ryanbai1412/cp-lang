// expect: ok, output "root 0 7 nodes"
use std.bitset.{BitTrie, BitTrieArena}
var ar = BitTrieArena<3>()
var t = BitTrie<3>.of(mut ar, 5)
t.add(mut ar, 1)
t.add(mut ar, 5)
print(t, ar)

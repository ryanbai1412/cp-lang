// expect: ok, output "4 nodes"
use std.bitset.{BitTrie, BitTrieArena}
var ar = BitTrieArena<3>()
BitTrie<3>.of(mut ar, 5)
print(ar)

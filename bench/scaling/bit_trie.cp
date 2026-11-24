// fixed: W
use std.bitset.{BitTrie, BitTrieArena}
const MUL = 2654435761
let n: Int = read()
var acc = 0
var ar = BitTrieArena<40>()
var t = BitTrie<40>.new()
for i in 0..n:
    t.add(mut ar, i * MUL % (1 << 40)) // timed: bitset.BitTrie.add
for i in 0..6 * n:
    acc += BitTrieArena<40>.new().node_count // timed: bitset.BitTrieArena.new
    acc += ar.node_count // timed: bitset.BitTrieArena.node_count
    acc += BitTrie<40>.new().len(ar) // timed: bitset.BitTrie.new
    acc += t.len(ar) // timed: bitset.BitTrie.len
    if t.is_empty: acc += 1 // timed: bitset.BitTrie.is_empty
for i in 0..2 * n:
    let x = i * MUL % (1 << 40)
    acc += BitTrie<40>.of(mut ar, x).len(ar) // timed: bitset.BitTrie.of
    acc += t.count(ar, x) // timed: bitset.BitTrie.count
for i in 0..n / 4:
    for s in t.walk(ar, i * MUL % (1 << 40)): acc += s.count // timed: bitset.BitTrie.walk
for x in t.each(ar): acc += x % 7 // timed: bitset.BitTrie.each
for r in 0..2:
    var a = BitTrie<40>.new()
    var b = BitTrie<40>.new()
    for i in 0..n:
        a.add(mut ar, i * MUL % (1 << 40))
        b.add(mut ar, i * 40503 % (1 << 40))
    a.absorb(mut ar, mut b) // timed: bitset.BitTrie.absorb
    acc += a.len(ar)
print(acc)

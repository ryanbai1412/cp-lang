// fixed: W
use std.bitset.{BitTrie}
let n: Int = read()
var acc = 0
var t = BitTrie<40>.new()
for i in 0..n:
    t.add(i * 2654435761 % (1 << 40)) // timed: bitset.BitTrie.add
for i in 0..6 * n:
    acc += BitTrie<40>.new().len // timed: bitset.BitTrie.new
    acc += t.len // timed: bitset.BitTrie.len
    if t.is_empty: acc += 1 // timed: bitset.BitTrie.is_empty
for i in 0..2 * n:
    let x = i * 2654435761 % (1 << 40)
    acc += BitTrie<40>.of(x).len // timed: bitset.BitTrie.of
    acc += t.count(x) // timed: bitset.BitTrie.count
for i in 0..n / 4:
    for s in t.walk(i * 2654435761 % (1 << 40)): acc += s.count // timed: bitset.BitTrie.walk
for r in 0..2:
    var a = BitTrie<40>.new()
    var b = BitTrie<40>.new()
    for i in 0..n:
        a.add(i * 2654435761 % (1 << 40))
        b.add(i * 40503 % (1 << 40))
    a.absorb(mut b) // timed: bitset.BitTrie.absorb
    acc += a.len
print(acc)

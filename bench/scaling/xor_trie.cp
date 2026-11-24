// fixed: bits
use std.bitset.{XorTrie}
let n: Int = read()
var acc = 0
for i in 0..6 * n:
    acc += XorTrie().len // timed: bitset.XorTrie.new
var t = XorTrie()
for i in 0..n:
    let x = i * 2654435761 % (1 << 40)
    t.add(x) // timed: bitset.XorTrie.add
for i in 0..6 * n:
    acc += t.len // timed: bitset.XorTrie.len
    if t.is_empty: acc += 1 // timed: bitset.XorTrie.is_empty
    t.xor_all(i % 8) // timed: bitset.XorTrie.xor_all
for i in 0..n:
    let x = i * 2654435761 % (1 << 40)
    if t.contains(x + 1): acc += 1 // timed: bitset.XorTrie.contains
    acc += t.count(x) // timed: bitset.XorTrie.count
    acc += t.max_xor(x) ?? 0 // timed: bitset.XorTrie.max_xor
    acc += t.min_xor(x + 1) ?? 0 // timed: bitset.XorTrie.min_xor
    acc += t.count_xor_less(x, 1 << 39) // timed: bitset.XorTrie.count_xor_less
    acc += t.kth_xor(x, i / 2) ?? 0 // timed: bitset.XorTrie.kth_xor
    acc += t.mex_xor(x) // timed: bitset.XorTrie.mex_xor
for i in 0..n:
    if t.remove(i * 2654435761 % (1 << 40)): acc += 1 // timed: bitset.XorTrie.remove
for r in 0..256:
    var u = XorTrie()
    u.reserve(64 * n) // timed: bitset.XorTrie.reserve
    acc += u.len
var u = XorTrie()
for r in 0..6 * n:
    u.add(r)
    u.clear() // timed: bitset.XorTrie.clear
    acc += u.len
print(acc)

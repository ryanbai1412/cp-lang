// n: 2048
// Interner looks keys up in a hash table, whose probes land anywhere in it. At n = 2048
// the 4n table (about 300 KB with its key list) still fits in one core's L2, so the O(1)
// lookups are timed in one cache tier.
use std.marks.{Interner}
let n: Int = read()
let reps = 1 << 18
var s = 0
var t = Interner<Int>()
for i in 0..reps:
    s += t.id(i * 7919 % n) // timed: marks.Interner.id
    s += t.get(i * 13 % n) ?? 0 // timed: marks.Interner.get
    if t.contains(i * 17 % n): s += 1 // timed: marks.Interner.contains
    s += t.key(i % t.len) // timed: marks.Interner.key
    s += t.len // timed: marks.Interner.len
    if t.is_empty: s += 1 // timed: marks.Interner.is_empty
for i in 0..reps:
    t.id(i)
    t.clear() // timed: marks.Interner.clear
print(s)

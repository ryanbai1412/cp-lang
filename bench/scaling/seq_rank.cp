// n: 5
// A rank fits an Int only for |V| <= 20: these time at 5 and 20.
use std.seq.{perm_unrank}
let n: Int = read()
domain V = 0..n
var p: [V]V = [V] v => v
for k in 0..n:
    p[V.nth(k)] = V.nth(n - 1 - k)
var s = 0
for r in 0..200000:
    s += p.perm_rank() // timed: seq.perm_rank
    s += Int(perm_unrank<V>(r % 100)[V.nth(0)] == V.nth(0)) // timed: seq.perm_unrank
print(s)

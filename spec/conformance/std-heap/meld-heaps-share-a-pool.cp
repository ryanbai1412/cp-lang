// expect: ok, output "1 2 3 4\n0\n15 16"
// rule: std.heap.MeldHeaps.meld
// Heaps in one `MeldHeaps` pool meld by relinking nodes; offsets stay with the heap they were added to.
use std.heap.{Meld, MeldHeaps}
domain V = 0..3
var p = MeldHeaps<Int>()
var hs: [V]Meld<Int> = Meld()
for v in V: hs[v] = p.of([v.ord * 2 + 1, v.ord * 2 + 2])
var all = p.meld(hs[0], hs[1])
print(p.drain(mut all))
print(all.len)
p.add_all(hs[2], 10)
print(p.drain(mut hs[2]))

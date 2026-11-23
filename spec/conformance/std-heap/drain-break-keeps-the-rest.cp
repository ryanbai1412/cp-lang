// expect: ok, output "2"
// `drain` pops one element per step, so leaving the loop early keeps the rest in the heap (round 22, B2).
var h = Heap.of([3, 1, 2])
for x in h.drain():
    if x == 1: break
print(h.len)

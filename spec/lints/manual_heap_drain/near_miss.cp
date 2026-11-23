// A loop that pushes back into the heap it pops is a simulation, not a drain.
let (n, k): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
var h = MinHeap<Int>()
for x in a: h.push(x)
for _ in 0..min(k, n - 1):
    let x = h.pop()!
    let y = h.pop()!
    h.push(x + y)
print(h.pop()!)
// Nor is one that calls a helper refilling the heap.
var g = MaxHeap<Int>()
fn refill(x: Int):
    if x > 1: g.push(x / 2)
for x in a: g.push(x)
var total = 0
for _ in 0..k:
    let x = g.pop()!
    total += x
    refill(x)
print(total)

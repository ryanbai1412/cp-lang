// Two calls with effects in one expression, before any assignment has been compiled:
// the operands are evaluated left to right.
let n: 1..=9 = read()
var heap = MinHeap<Int>()
for k in 0..n:
    let x: 1..=99 = read()
    heap.push(x)
let merged = heap.pop()! * 100 + heap.pop()!
print(merged, heap.len())

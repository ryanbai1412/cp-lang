use std.heap.{MinHeap}

let (n, k): (1..=100, 1..=100) = read()
var pq = MinHeap<Int>()
for _ in 0..n:
    let x: Int = read()
    pq.push(x)

var picked: List<Int> = []
for _ in 0..min(n, k):
    picked.push(pq.pop()!)
print(sum(picked))

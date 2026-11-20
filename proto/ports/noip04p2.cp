// NOIP 2004 Merging Fruit: merging two piles costs their combined size; the
// cheapest order always merges the two smallest piles (Huffman).
let n: 1..=10000 = read()
let piles: [0..n]1..=20000 = read()

var heap = MinHeap<Int>()
for p in piles: heap.push(p)
var effort = 0
while heap.len > 1:
    let merged = heap.pop()! + heap.pop()!
    effort += merged
    heap.push(merged)
print(effort)

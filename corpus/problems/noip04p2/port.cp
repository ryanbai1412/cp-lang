// Source: NOIP 2004 Senior P2 (Combine Fruit), https://dmoj.ca/problem/noip04p2
// Problem: n <= 10000 piles of fruit, weights <= 20000. Merging two piles costs their total
//   weight; print the minimum total cost to merge everything into one pile.
// Solution: Huffman: always merging the two lightest piles is optimal. A MinHeap pops two,
//   adds their sum to the cost and pushes it back, n - 1 times: O(n log n).
// original: corpus/submissions/5117099.cpp
let n: 1..=10000 = read()
let piles: [0..n]1..=20000 = read()

var heap = Heap.of(piles.collect())
var effort = 0
while heap.len > 1:
    let merged = heap.pop()! + heap.pop()!
    effort += merged
    heap.push(merged)
print(effort)

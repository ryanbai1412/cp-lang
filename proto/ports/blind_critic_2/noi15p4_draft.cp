// NOI 2015 Homer's Epic: k-ary prefix code for n words with frequencies w, minimising
// the total length, then the longest codeword.
// DRAFT 1: k-ary Huffman, merging the k lightest subtrees each round.
let n: 2..=1e5 = read()
let k: 2..=9 = read()
let w: [n]1..=1e11 = read()

var heap = MinHeap<(Int, Int)>()
for x in w: heap.push((x, 0))
var total = 0
while heap.len() > 1:
    var weight = 0
    var depth = 0
    for _ in 0..k:
        if let (x, d) = heap.pop():
            weight += x
            depth max= d
    total += weight
    heap.push((weight, depth + 1))
print(total)
print(heap.pop()!.1)

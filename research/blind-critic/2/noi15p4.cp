// NOI 2015 Homer's Epic: k-ary prefix code for n words with frequencies w, minimising
// the total length, then the longest codeword.
// k-ary Huffman: merge the k lightest subtrees each round (ties: the shallowest first), after
// padding with weight-0 words so that every merge, the last one included, takes exactly k.
let n: 2..=1e5 = read()
let k: 2..=9 = read()
let w: [n]1..=1e11 = read()

var heap = MinHeap<(Int, Int)>()
for x in w: heap.push((x, 0))
while (heap.len - 1) % (k - 1) != 0: heap.push((0, 0))
var total = 0
while heap.len > 1:
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

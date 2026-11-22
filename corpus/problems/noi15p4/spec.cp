// NOI 2015 "Homer's Epic": k-ary Huffman code, minimizing total length and then
// the longest word. Pad with weight-0 leaves so every merge takes exactly k.
let (n, k) = read<(1..=1e5, 2..=9)>()
let w = read<[n]0..=1e11>()
let pad = (k - 1 - (n - 1) % (k - 1)) % (k - 1)
var heap = MinHeap.of(w.map(x => (w: x, depth: 0)).collect() + [(w: 0, depth: 0) for _ in 0..pad])
var total = 0
while heap.len > 1:
    let kids = [heap.pop()! for _ in 0..k]
    let merged = (w: kids.sum(of: .w), depth: kids.max(of: .depth) + 1)
    total += merged.w
    heap.push(merged)
print(total)
print(heap.pop()!.depth)

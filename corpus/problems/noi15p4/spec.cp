// noi15p4 Homer's Epic: a k-ary prefix code minimising total weighted length, then the longest word.
// k-ary Huffman: pad with zero-weight leaves until every merge takes exactly k, and merge the
// lightest (then shallowest) k each time.
let (n, k) = read<(2..=100_000, 2..=9)>()
var heap = MinHeap<(w: Int, depth: Int)>(by: x => (x.w, x.depth))
for w in read<[n]1..=100_000_000_000>():
    heap.push((w: w, depth: 0))
while (heap.len - 1) % (k - 1) != 0:
    heap.push((w: 0, depth: 0))
var total = 0
while heap.len > 1:
    let picked = List((0..k).map(_ => heap.pop()!))
    let merged = (w: picked.map(x => x.w).sum(), depth: picked.map(x => x.depth).max()! + 1)
    total += merged.w
    heap.push(merged)
print(total)
print(heap.pop()!.depth)

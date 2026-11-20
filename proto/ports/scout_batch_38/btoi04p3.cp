// Baltic OI '04 P3 (Sequence): find a strictly increasing sequence b
// minimizing sum(|a[i] - b[i]|). Shifting x[i] = a[i] - i reduces to L1
// isotonic regression (Pool-Adjacent-Violators with mergeable lower-median
// max-heaps, replacing the original's 130-line pointer skew heap).
// original: corpus/submissions/5270736.cpp
let n: 1..=1e6 = read()
domain V = 0..n
let a: [V]Int = read()
let x: [V]Int = [V] i => a[i] - i.to_idx

struct Block:
    lo: V
    hi: V
    len: Int
    heap: MaxHeap<Int>

    fn med(self) -> Int: self.heap.peek!

fn merge_blocks(mut l: Block, mut r: Block) -> Block:
    // FRICTION 38.8: small-to-large heap merge has no .absorb(mut other)
    // or meldable heap in std, so we swap + drain manually.
    if l.heap.len < r.heap.len: swap(mut l.heap, mut r.heap)
    while let v = r.heap.pop(): l.heap.push(v)
    let new_len = l.len + r.len
    while l.heap.len > (new_len + 1) / 2:
        l.heap.pop()
    Block(lo: l.lo, hi: r.hi, len: new_len, heap: l.heap)

var stack: List<Block> = []
for i in V:
    var h = MaxHeap<Int>()
    h.push(x[i])
    var cur = Block(lo: i, hi: i, len: 1, heap: h)
    while let top = stack.last and top.med() > cur.med():
        var prev = stack.pop()!
        cur = merge_blocks(mut prev, mut cur)
    stack.push(cur)

var cost = 0
for blk in stack:
    let m = blk.med()
    for i in blk.lo..=blk.hi: cost += abs(x[i] - m)

print(cost)
for blk in stack:
    let m = blk.med()
    for i in blk.lo..=blk.hi: print(m + i.to_idx)

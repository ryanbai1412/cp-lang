// Source: DMOPC '20 Contest 7 P1 (Bob and Balance), https://dmoj.ca/problem/dmopc20c7p1
// Problem: 2n weights (n <= 5e5, masses <= 1e9) go on n two-plate scales, one weight per plate.
//   Maximise the tipped scales (unequal masses); print that number, then each scale's two
//   weight indices.
// Solution: greedy: repeatedly pair the two masses with the most weights left (a max-heap on
//   count); a mass is paired with itself only when it is the last one left. Weights of each
//   mass are compressed and listed, and popped to print the pairs. O(n log n).
// original: corpus/submissions/3686504.cpp
let n: 1..=1e5 = read()
domain Slot = 1..=2 * n
let kind: [Slot]1..=1e9 = read()

let (K, name) = compress(kind)
var slots: [K]List<Slot> = []  // where the problems of each kind are
for s in Slot: slots[name.first_ge(kind[s])!].push(s)

// Pair the two kinds that have the most problems left, again and again; a
// kind is paired with itself only when it is the last one left.
var left = MinHeap<(Int, Int)>()  // (-count, -kind): the largest count is on top
fn put_back(count: Int, k: Int):
    if count < 0: left.push((count, k))
for k in K: put_back(-slots[k].len, -k.idx)
var pairs: List<(K, K)> = []
for _ in 0..n:
    let (c1, k1) = left.pop()!
    if let (c2, k2) = left.pop():
        pairs.push((K(-k1), K(-k2)))
        put_back(c1 + 1, k1)
        put_back(c2 + 1, k2)
    else:
        pairs.push((K(-k1), K(-k1)))
        put_back(c1 + 2, k1)
print(count(a != b for (a, b) in pairs))
for (a, b) in pairs: print(slots[a].pop()!, slots[b].pop()!)

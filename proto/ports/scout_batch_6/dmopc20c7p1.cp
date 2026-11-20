// DMOPC '20 Contest 7 P1 Bob and Balance: 2n problems each have a kind. Split
// them into n pairs so that as many pairs as possible hold two different
// kinds. Print that number, then the pairs.
// original: corpus/submissions/3686504.cpp
let n: 1..=1e5 = read()
domain Slot = 1..=2 * n
let kind: [Slot]1..=1e9 = read()
var slots = group(Slot, by: s => kind[s])           // where the problems of each kind are     WISH S15

// Pair the two kinds that have the most problems left, again and again; a
// kind is paired with itself only when it is the last one left.
var left = MaxHeap<(count: Int, kind: Int)>()                                       // WISH S11 (B5)
fn put_back(count: Int, k: Int):
    if count > 0: left.push((count, k))
for (k, at) in slots: put_back(at.len, k)
var pairs: List<(Int, Int)> = []
for _ in 0..n:
    let (c1, k1) = left.pop()!
    if let (c2, k2) = left.pop():
        pairs.push((k1, k2))
        put_back(c1 - 1, k1)
        put_back(c2 - 1, k2)
    else:
        pairs.push((k1, k1))
        put_back(c1 - 2, k1)
print(count(a != b for (a, b) in pairs))
for (a, b) in pairs: print(slots[a].pop()!, slots[b].pop()!)
